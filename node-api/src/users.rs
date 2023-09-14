use crate::consts;
use pwhash::sha512_crypt;
use regex::Regex;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::process::Command;
use time::{macros::format_description, Date};

use self::models::*;

pub mod models {
    use serde::{Deserialize, Serialize};

    #[derive(Deserialize, Serialize)]
    pub struct SSHUserInfo {
        pub username: String,
        pub userid: u32,
        pub usergroup: Option<(u32, String)>,
        pub exp_date: String,
    }

    #[derive(Deserialize, Serialize)]
    pub struct UserRawCreds {
        pub username: String,
        pub password: String,
        pub password_hash: String,
    }

    #[derive(Deserialize, Serialize)]
    pub struct UserExp {
        pub username: String,
        pub exp_date: String,
    }

    #[derive(Deserialize, Serialize)]
    pub struct UserStatus {
        pub username: String,
        pub status: String,
    }

    #[derive(Deserialize, Serialize)]
    pub struct ChExpMsg {
        pub username: String,
        pub exp_date: String,
        pub message: String,
    }

    #[derive(Deserialize, Serialize)]
    pub struct ChGrpMsg {
        pub username: String,
        pub group: String,
        pub message: String,
    }

    #[derive(Deserialize, Serialize)]
    pub enum UserErrors {
        UserAlreadyExists,
        InvalidUserOrGroup,
        InvalidShell,
        InvalidExpDate,
        InvalidPasswordHash,
        PermissionDenied,
        UnexpectedError,
        ProcessTerminated,
        CannotDeleteYourSelf,
        InvalidTraceFile,
        CommandNotFound,
    }
}

//NOTE: Deprecated should be removed in newer versions
#[derive(Deserialize, Serialize, Debug)]
pub struct UserCredentials {
    pub username: String,
    pub password_hash: String,
}

impl UserCredentials {
    pub fn new(username: String) -> UserCredentials {
        UserCredentials {
            username: username.clone(),
            password_hash: Self::hash_password(&Self::gen_password(username)),
        }
    }

    pub fn new_raw(username: String, password: String) -> UserCredentials {
        // Verify password hash formatting
        UserCredentials {
            username,
            password_hash: Self::hash_password(&password),
        }
    }

    pub fn hash_password(password: &str) -> String {
        sha512_crypt::hash_with(consts::PASSWD_PARAMS, password).unwrap()
    }

    pub fn gen_password(username: String) -> String {
        //TODO: just randomly generate it
        let mut hasher = Sha256::new();

        hasher.update(consts::PASSWD_IV);
        hasher.update(username.as_bytes());

        let hash = hasher.finalize();

        let hex_hash = format!("{hash:0x}");
        format!("{}{}", consts::PASSWD_PREFIX, &hex_hash[10..16])
    }

    pub fn get_username(&self) -> &str {
        &self.username
    }
    pub fn get_password_hash(&self) -> &str {
        &self.password_hash
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SSHUser {
    #[serde(flatten)]
    pub user_credentials: UserCredentials,
    pub shell: String,
    pub usergroup: String,
    pub exp_date: String,
}

impl SSHUser {
    pub fn auto_add(
        users_info: (&str, u64),
        usergroup: String,
        exp_date: String,
    ) -> Result<SSHUser, UserErrors> {
        //NOTE: users_info contains username's prefix and users count
        let username = format!("{}{}", users_info.0, users_info.1 + 1);
        let password = UserCredentials::gen_password(username.clone());

        Self::add(
            username,
            consts::DEFAULT_SHELL.to_string(),
            usergroup,
            exp_date,
            password,
        )
    }

    pub fn add(
        username: String,
        shell: String,
        usergroup: String,
        exp_date: String,
        password: String,
    ) -> Result<SSHUser, UserErrors> {
        let exp_date = Self::format_exp_date(&exp_date)?;
        let user_credentials = UserCredentials::new_raw(username, password);

        let process_status = Command::new("useradd")
            .arg("-p")
            .arg(user_credentials.get_password_hash())
            .arg("-s")
            .arg(&shell)
            .arg("-g")
            .arg(&usergroup)
            .arg("-e")
            .arg(&exp_date)
            .arg(user_credentials.get_username())
            .status();

        match process_status {
            Ok(status) => {
                if let Some(error) = Self::unixuser_code_to_err(status.code()) {
                    Err(error)
                } else {
                    Ok(SSHUser {
                        user_credentials,
                        shell,
                        usergroup,
                        exp_date,
                    })
                }
            }
            Err(_) => Err(UserErrors::CommandNotFound),
        }
    }

    pub fn restore_password(username: String) -> UserRawCreds {
        let password = UserCredentials::gen_password(username.clone());
        let password_hash = UserCredentials::hash_password(&password);

        UserRawCreds {
            username,
            password,
            password_hash,
        }
    }

    pub fn del(&self) -> Result<UserStatus, UserErrors> {
        Self::userdel(self.user_credentials.get_username())
    }

    pub fn userdel(username: &str) -> Result<UserStatus, UserErrors> {
        let process_status = Command::new("userdel").arg(username).status();

        match process_status {
            Ok(status) => {
                if let Some(error) = Self::unixuser_code_to_err(status.code()) {
                    Err(error)
                } else {
                    Ok(UserStatus {
                        username: username.to_string(),
                        status: format!("user {username} sucessfully deleted"),
                    })
                }
            }
            Err(_) => Err(UserErrors::CommandNotFound),
        }
    }

    pub fn usermod_change_pass(username: &str, password: &str) -> Result<UserRawCreds, UserErrors> {
        let password_hash = UserCredentials::hash_password(password);

        let process_status = Command::new("usermod")
            .arg("-p")
            .arg(&password_hash)
            .arg(username)
            .status();

        match process_status {
            Ok(status) => {
                if let Some(error) = Self::unixuser_code_to_err(status.code()) {
                    Err(error)
                } else {
                    Ok(UserRawCreds {
                        username: username.to_string(),
                        password: password.to_string(),
                        password_hash,
                    })
                }
            }
            Err(_) => Err(UserErrors::CommandNotFound),
        }
    }

    pub fn chexp(&self, exp_date: &str) -> Result<ChExpMsg, UserErrors> {
        Self::usermod_change_exp(self.user_credentials.get_username(), exp_date)
    }

    pub fn usermod_change_exp(username: &str, exp_date: &str) -> Result<ChExpMsg, UserErrors> {
        let exp_date = Self::format_exp_date(exp_date)?;

        let process_status = Command::new("chage")
            .arg("-E")
            .arg(&exp_date)
            .arg(username)
            .status();

        match process_status {
            Ok(status) => {
                if let Some(error) = Self::unixuser_code_to_err(status.code()) {
                    Err(error)
                } else {
                    Ok(ChExpMsg {
                        username: username.to_string(),
                        exp_date: exp_date.to_string(),
                        message: format!(
                            "user {username}'s expiry date sucessfully changed to {exp_date}"
                        ),
                    })
                }
            }
            Err(_) => Err(UserErrors::CommandNotFound),
        }
    }

    pub fn chgrp(&self, group: &str) -> Result<ChGrpMsg, UserErrors> {
        Self::usermod_change_grp(self.user_credentials.get_username(), group)
    }

    pub fn usermod_change_grp(username: &str, group: &str) -> Result<ChGrpMsg, UserErrors> {
        let process_status = Command::new("usermod")
            .arg("-g")
            .arg(group)
            .arg(username)
            .status();

        match process_status {
            Ok(status) => {
                if let Some(error) = Self::unixuser_code_to_err(status.code()) {
                    Err(error)
                } else {
                    Ok(ChGrpMsg {
                        username: username.to_string(),
                        group: group.to_string(),
                        message: format!("user {username}'s group sucessfully changed to {group}"),
                    })
                }
            }
            Err(_) => Err(UserErrors::CommandNotFound),
        }
    }

    pub fn lock(&self) -> Result<UserStatus, UserErrors> {
        Self::usermod_lock(self.user_credentials.get_username())
    }

    pub fn usermod_lock(username: &str) -> Result<UserStatus, UserErrors> {
        let process_status = Command::new("usermod").arg("-L").arg(username).status();

        match process_status {
            Ok(status) => {
                if let Some(error) = Self::unixuser_code_to_err(status.code()) {
                    Err(error)
                } else {
                    Ok(UserStatus {
                        username: username.to_string(),
                        status: format!("user {username} sucessfully locked"),
                    })
                }
            }
            Err(_) => Err(UserErrors::CommandNotFound),
        }
    }

    pub fn unlock(&self) -> Result<UserStatus, UserErrors> {
        Self::usermod_unlock(self.user_credentials.get_username())
    }

    pub fn usermod_unlock(username: &str) -> Result<UserStatus, UserErrors> {
        let process_status = Command::new("usermod").arg("-U").arg(username).status();

        match process_status {
            Ok(status) => {
                if let Some(error) = Self::unixuser_code_to_err(status.code()) {
                    Err(error)
                } else {
                    Ok(UserStatus {
                        username: username.to_string(),
                        status: format!("user {username} sucessfully unlocked"),
                    })
                }
            }
            Err(_) => Err(UserErrors::CommandNotFound),
        }
    }

    pub fn exp(&self) -> Result<UserExp, UserErrors> {
        Self::get_chage_exp(self.user_credentials.get_username())
    }

    // TODO: clean it up
    pub fn get_chage_exp(username: &str) -> Result<UserExp, UserErrors> {
        let process_output = Command::new("chage").arg("-l").arg(username).output();
        match process_output {
            Ok(output) => {
                if output.status.code() != Some(0) {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    if stderr.contains("does not exist in /etc/passwd") {
                        Err(UserErrors::InvalidUserOrGroup)
                    } else {
                        Err(Self::unixuser_code_to_err(output.status.code()).unwrap())
                    }
                } else {
                    let user_info = String::from_utf8_lossy(&output.stdout);

                    let re = Regex::new("Account expires\t+: (.*)\n").unwrap();

                    let caps = re.captures(&user_info);

                    if let Some(caps) = caps {
                        let exp_date = caps.get(1).map_or("", |m| m.as_str()).to_string();
                        if exp_date == "never" {
                            return Ok(UserExp {
                                username: username.to_string(),
                                exp_date: "never".to_string(),
                            });
                        }

                        let inp_format = format_description!("[month repr:short] [day], [year]");

                        let out_format = format_description!("[year]-[month]-[day]");
                        match Date::parse(&exp_date, &inp_format) {
                            Ok(date) => Ok(UserExp {
                                username: username.to_string(),
                                exp_date: date.format(&out_format).unwrap(),
                            }),
                            Err(_) => Err(UserErrors::InvalidExpDate),
                        }
                    } else {
                        Err(UserErrors::UnexpectedError)
                    }
                }
            }
            Err(_e) => Err(UserErrors::CommandNotFound),
        }
    }

    pub fn get_user(username: &str) -> Option<SSHUserInfo> {
        match users::get_user_by_name(username) {
            Some(user) => {
                let group_id = user.primary_group_id();
                let usergroup: Option<(u32, String)> = users::get_group_by_gid(group_id)
                    .map(|group| (group.gid(), group.name().to_string_lossy().to_string()));

                let exp_info = Self::get_chage_exp(username).unwrap_or(UserExp {
                    username: username.to_string(),
                    exp_date: "N/A".to_string(),
                });

                Some(SSHUserInfo {
                    username: username.to_string(),
                    userid: user.uid(),
                    usergroup,
                    exp_date: exp_info.exp_date,
                })
            }

            None => None,
        }
    }

    pub fn get_users_by_prefix(prefix: &str) -> Vec<String> {
        Self::get_users_core(prefix, None)
    }

    pub fn get_users_by_group(usergroup: &str) -> Vec<String> {
        Self::get_users_core("", Some(usergroup))
    }

    pub fn get_users_core(prefix: &str, usergroup: Option<&str>) -> Vec<String> {
        let iter = unsafe { users::all_users() };
        let mut users_list: Vec<String> = Vec::new();

        for user in iter {
            let username = user.name().to_string_lossy();
            let groups = user.groups();

            if username.starts_with(prefix) {
                if let Some(usergroup) = usergroup {
                    if let Some(groups) = groups {
                        let group = groups.iter().find(|g| g.name() == usergroup);
                        if group.is_some() {
                            users_list.push(username.to_string());
                        }
                    }
                } else {
                    users_list.push(username.to_string());
                }
            }
        }

        users_list
    }

    pub fn get_usage_by_group(usergroup: &str) -> Result<HashMap<String, f64>, UserErrors> {
        let users = Self::get_users_by_group(usergroup);
        Self::get_usage_core(&users)
    }

    pub fn get_usage_by_name(username: &str) -> Result<HashMap<String, f64>, UserErrors> {
        Self::get_usage_core(&vec![username.to_string()])
    }

    pub fn get_usage_by_prefix(prefix: &str) -> Result<HashMap<String, f64>, UserErrors> {
        let users = Self::get_users_by_prefix(prefix);
        Self::get_usage_core(&users)
    }

    // NOTE: Needs Cleaning up and optimization and error handling
    pub fn get_usage_core(users: &Vec<String>) -> Result<HashMap<String, f64>, UserErrors> {
        let binding = match std::fs::read_to_string(consts::NETHOGS_TRACE_PATH) {
            Ok(binding) => binding,
            Err(_e) => return Err(UserErrors::InvalidTraceFile),
        };
        let log_content = binding.split('\n');

        let re = Regex::new(r".+/.+/\w+[\s|\t]+(.+)").unwrap();

        let mut usage_list: HashMap<String, f64> = HashMap::new();

        for user in users.iter() {
            let mut total_usage = 0f64;
            for log_line in log_content.clone() {
                if log_line.contains(user) {
                    let caps = re
                        .captures(&log_line)
                        .unwrap_or(re.captures("a: n/0/0  0  0").unwrap());
                    let usage_part = caps.get(1).map_or("", |m| m.as_str()).to_string();
                    for usage in usage_part.split('\t') {
                        total_usage += usage.parse::<f64>().unwrap_or(0f64);
                    }
                }
            }
            usage_list.insert(user.to_string(), total_usage);
        }
        Ok(usage_list)
    }

    fn format_exp_date(exp_date: &str) -> Result<String, UserErrors> {
        let format = format_description!("[year]-[month]-[day]");
        match Date::parse(exp_date, &format) {
            Ok(date) => Ok(date.format(&format).unwrap()),
            Err(_) => Err(UserErrors::InvalidExpDate),
        }
    }

    fn unixuser_code_to_err(code: Option<i32>) -> Option<UserErrors> {
        if let Some(code) = code {
            match code {
                0 => None,
                1 => Some(UserErrors::PermissionDenied),
                3 => Some(UserErrors::InvalidShell),
                6 => Some(UserErrors::InvalidUserOrGroup),
                9 => Some(UserErrors::UserAlreadyExists),
                _ => Some(UserErrors::UnexpectedError),
            }
        } else {
            Some(UserErrors::ProcessTerminated)
        }
    }
}

