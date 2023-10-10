use crate::consts;
use rocket::serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct SSHUser {
    pub username: String,
    pub password_hash: String,
    pub shell: String,
    pub usergroup: String,
    pub exp_date: String,
}

#[derive(Deserialize, Serialize)]
pub struct InputSSHUser {
    pub username: String,
    pub password: String,
    pub exp_date: String,
    pub group: String,
    pub shell: Option<String>,
}

impl InputSSHUser {
    pub fn auto_gen(max_logins: i32, user_id: i32, days: Option<i64>) -> InputSSHUser {
        let username = format!("{0}{max_logins}x{user_id:03}", consts::PREFIX);
        let group = format!("{0}{max_logins}", consts::GROUP_PREFIX);
        let password = crate::gen_password();

        let exp_days = days.unwrap_or(30);
        let exp_date = crate::add_to_time(exp_days);

        InputSSHUser {
            username,
            password,
            exp_date,
            group,
            shell: None,
        }
    }
}

