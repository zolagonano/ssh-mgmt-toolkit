#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

use lib::config;
use lib::consts;
use lib::models::*;
use lib::stats::*;
use lib::users::models::*;
use lib::users::*;
use rocket::State;
use rocket_contrib::json::Json;
use serde_json::json;
use serde_json::Value;
use std::collections::HashMap;

macro_rules! api_err {
    ($msg:expr) => {
        Json(json!({ "Err": $msg }))
    };
}

#[get("/ping")]
fn ping() -> &'static str {
    "pong"
}

#[get("/node_info")]
fn node_info(node_config: State<config::ConfigFile>) -> Json<config::NodeInfo> {
    Json(node_config.node_info.clone())
}

#[get("/net_stats")]
fn net_stats() -> Json<Vec<NetworkUsage>> {
    Json(NetworkUsage::get_usage())
}

#[get("/hw_stats")]
fn hw_stats() -> Json<HwUsage> {
    Json(HwUsage::new())
}

#[post("/list_users", format = "json", data = "<lookup_params>")]
fn list_users(
    token: Token,
    lookup_params: Json<UserLookupParams>,
) -> Result<Json<Vec<String>>, Json<Value>> {
    token.validate()?;

    if lookup_params.prefix.is_none() && lookup_params.group.is_none() {
        return Err(api_err!("Please provide username's prefix or groupname"));
    }

    if let Some(prefix) = &lookup_params.prefix {
        return Ok(Json(SSHUser::get_users_by_prefix(&prefix)));
    } else {
        if let Some(group) = &lookup_params.group {
            return Ok(Json(SSHUser::get_users_by_group(&group)));
        } else {
            return Ok(Json(SSHUser::get_users_by_prefix("")));
        }
    }
}

#[get("/user_expiry/<user>")]
fn user_expiry(
    token: Token,
    user: String,
) -> Result<Json<Result<UserExp, UserErrors>>, Json<Value>> {
    token.validate()?;
    Ok(Json(SSHUser::get_chage_exp(&user)))
}

#[post("/userdel", format = "json", data = "<lookup_params>")]
fn userdel(
    token: Token,
    lookup_params: Json<UserLookupParams>,
) -> Result<Json<Result<UserStatus, UserErrors>>, Json<Value>> {
    match token.validate() {
        Ok(role) => match role {
            Role::Privileged => {
                if let Some(username) = &lookup_params.username {
                    Ok(Json(SSHUser::userdel(&username)))
                } else {
                    Err(api_err!("username field cannot be empty"))
                }
            }
            _ => Err(api_err!("Authentication Failed")),
        },
        Err(e) => Err(api_err!(*e)),
    }
}

#[post("/auto_useradd", format = "json", data = "<user_data>")]
fn auto_useradd(
    token: Token,
    user_data: Json<AutoSSHUser>,
) -> Result<Json<Result<SSHUser, UserErrors>>, Json<Value>> {
    match token.validate() {
        Ok(role) => match role {
            Role::Privileged => {
                let user_data = user_data.0;
                Ok(Json(SSHUser::auto_add(
                    (&user_data.prefix, user_data.users_count),
                    user_data.group,
                    user_data.exp_date,
                )))
            }
            _ => Err(api_err!("Authentication Failed")),
        },
        Err(e) => Err(api_err!(*e)),
    }
}

#[post("/useradd", format = "json", data = "<user_data>")]
fn useradd(
    token: Token,
    user_data: Json<InputSSHUser>,
) -> Result<Json<Result<SSHUser, UserErrors>>, Json<Value>> {
    match token.validate() {
        Ok(role) => match role {
            Role::Privileged => {
                let user_data = user_data.0;
                let shell = user_data.shell.unwrap_or(consts::DEFAULT_SHELL.to_string());
                Ok(Json(SSHUser::add(
                    user_data.username,
                    shell,
                    user_data.group,
                    user_data.exp_date,
                    user_data.password,
                )))
            }
            _ => Err(api_err!("Authentication Failed")),
        },
        Err(e) => Err(api_err!(*e)),
    }
}

#[post("/passwd", format = "json", data = "<user>")]
fn passwd(
    token: Token,
    user: Json<UserPasswd>,
) -> Result<Json<Result<UserRawCreds, UserErrors>>, Json<Value>> {
    match token.validate() {
        Ok(role) => match role {
            Role::Privileged => Ok(Json(SSHUser::usermod_change_pass(
                &user.username,
                &user.password,
            ))),
            _ => Err(api_err!("Authentication Failed")),
        },
        Err(e) => Err(api_err!(*e)),
    }
}

#[post("/chgrp", format = "json", data = "<user>")]
fn chgrp(
    token: Token,
    user: Json<UserGrp>,
) -> Result<Json<Result<ChGrpMsg, UserErrors>>, Json<Value>> {
    match token.validate() {
        Ok(role) => match role {
            Role::Privileged => Ok(Json(SSHUser::usermod_change_grp(
                &user.username,
                &user.group,
            ))),
            _ => Err(api_err!("Authentication Failed")),
        },
        Err(e) => Err(api_err!(*e)),
    }
}

#[post("/chexp", format = "json", data = "<user>")]
fn chexp(
    token: Token,
    user: Json<UserExpDate>,
) -> Result<Json<Result<ChExpMsg, UserErrors>>, Json<Value>> {
    match token.validate() {
        Ok(role) => match role {
            Role::Privileged => Ok(Json(SSHUser::usermod_change_exp(
                &user.username,
                &user.exp_date,
            ))),
            _ => Err(api_err!("Authentication Failed")),
        },
        Err(e) => Err(api_err!(*e)),
    }
}

#[post("/userlock", format = "json", data = "<user>")]
fn userlock(
    token: Token,
    user: Json<OnlyUser>,
) -> Result<Json<Result<UserStatus, UserErrors>>, Json<Value>> {
    match token.validate() {
        Ok(role) => match role {
            Role::Privileged => Ok(Json(SSHUser::usermod_lock(&user.username))),
            _ => Err(api_err!("Authentication Failed")),
        },
        Err(e) => Err(api_err!(*e)),
    }
}

#[post("/userunlock", format = "json", data = "<user>")]
fn userunlock(
    token: Token,
    user: Json<OnlyUser>,
) -> Result<Json<Result<UserStatus, UserErrors>>, Json<Value>> {
    match token.validate() {
        Ok(role) => match role {
            Role::Privileged => Ok(Json(SSHUser::usermod_unlock(&user.username))),
            _ => Err(api_err!("Authentication Failed")),
        },
        Err(e) => Err(api_err!(*e)),
    }
}

//NOTE: needs working on
#[post("/users_usage", format = "json", data = "<lookup_params>")]
fn users_usage(
    token: Token,
    lookup_params: Json<UserLookupParams>,
) -> Result<Json<Result<HashMap<String, f64>, UserErrors>>, Json<Value>> {
    token.validate()?;

    if lookup_params.prefix.is_none()
        && lookup_params.group.is_none()
        && lookup_params.username.is_none()
    {
        return Err(api_err!(
            "Please provide username's prefix, groupname, or the username"
        ));
    }

    if let Some(prefix) = &lookup_params.prefix {
        return Ok(Json(SSHUser::get_usage_by_prefix(&prefix)));
    } else {
        if let Some(group) = &lookup_params.group {
            return Ok(Json(SSHUser::get_usage_by_group(&group)));
        } else {
            if let Some(username) = &lookup_params.username {
                return Ok(Json(SSHUser::get_usage_by_name(&username)));
            } else {
                return Ok(Json(SSHUser::get_usage_by_prefix("")));
            }
        }
    }
}

fn main() {
    rocket::ignite()
        .manage(config::ConfigFile::load().unwrap_or_else(|_| panic!("Couldn't load config file!")))
        .mount("/", routes![ping])
        .mount("/api", routes![node_info])
        .mount(
            "/api/stats",
            routes![
                ping,
                node_info,
                net_stats,
                hw_stats,
                list_users,
                user_expiry,
                users_usage,
            ],
        )
        .mount(
            "/api/cmd",
            routes![
                userdel,
                useradd,
                auto_useradd,
                passwd,
                chgrp,
                chexp,
                userlock,
                userunlock
            ],
        )
        .launch();
}

