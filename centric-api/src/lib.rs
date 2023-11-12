pub mod models;
pub mod node_models;
pub mod schema;
pub mod token;

pub mod consts {
    pub const ADMIN_KEY: &str = "THIS_IS_THE_ADMIN_KEY";
    pub const JWT_SECRET: &[u8] =
        b"5891b5b522d5df086d0ff0b110fbd9d21bb4fc7163af34d08286a2e846f6be03";

    pub const NODE_INFO_PATH: &str = "/api/stats/node_info";
    pub const HW_STATS_PATH: &str = "/api/stats/hw_stats";
    pub const NET_STATS_PATH: &str = "/api/stats/net_stats";
    pub const USERADD_PATH: &str = "/api/cmd/useradd";

    pub const PASS_PREFIX: &str = "SSHMGMTKIT_";
    pub const GROUP_PREFIX: &str = "grp";
    pub const PREFIX: &str = "sshmgmt";
}

use rocket_sync_db_pools::{database, diesel};

#[database("SSHMGMTCentricDB")]
pub struct Db(diesel::PgConnection);

use chrono::{Duration, Local, NaiveDate};
use pwhash::sha512_crypt;
use rand::prelude::*;

/// Hashes the given password using SHA-512 Crypt and returns the hashed password.
pub fn hash_password(password: &str) -> String {
    sha512_crypt::hash(password).unwrap()
}

/// Verifies if the given password matches the provided hash.
pub fn verify_hash(password: &str, hash: &str) -> bool {
    sha512_crypt::verify(password, hash)
}

/// Checks if the provided username is valid based on certain criteria.
pub fn is_valid_username(username: &str) -> bool {
    let username_len = username.len();

    if username_len < 3 || username_len > 20 {
        return false;
    }

    if !username
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_')
    {
        return false;
    }

    if let Some(c) = username.chars().next() {
        if c.is_ascii_digit() {
            return false;
        }
    }

    true
}

/// Generates a random password following a specific pattern.
pub fn gen_password() -> String {
    let mut rng = rand::thread_rng();
    let random_number = rng.gen_range(0..100000);
    format!("{0}{random_number:05}", consts::PASS_PREFIX)
}

/// Formats the expiry date string to a specific date format.
pub fn format_exp_date(exp_date: &str) -> Result<String, String> {
    if let Ok(date) = NaiveDate::parse_from_str(exp_date, "%Y-%m-%d") {
        Ok(date.format("%Y-%m-%d").to_string())
    } else {
        Err("Invalid expiry date".to_string())
    }
}

/// Adds the specified number of days to the current date and returns the formatted future date.
pub fn add_to_time(days: i64) -> String {
    let now = Local::now().naive_local().date();
    let future_date = now + Duration::days(days);
    let formatted_date = future_date.format("%Y-%m-%d").to_string();
    format!("{}", formatted_date)
}
