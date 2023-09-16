use rocket::http::Status;
use rocket::request::{self, FromRequest, Request};
use rocket::Outcome;
use serde::{Deserialize, Serialize};

use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};

use rocket_contrib::json::Json;
use serde_json::{json, Value};

use crate::consts::JWT_SECRET;

// TODO: Generate and verify Unix passwords with structs

// NOTE: Data models used in serializing request data should become a separated library in future
// because they will be needed in the telegram-bot as well

#[derive(Deserialize, Serialize)]
pub struct UserLookupParams {
    pub username: Option<String>,
    pub prefix: Option<String>,
    pub group: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct UserExpDate {
    pub username: String,
    pub exp_date: String,
}

#[derive(Deserialize, Serialize)]
pub struct UserPasswd {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize, Serialize)]
pub struct UserGrp {
    pub username: String,
    pub group: String,
}

#[derive(Deserialize, Serialize)]
pub struct OnlyUser {
    pub username: String,
}

#[derive(Deserialize, Serialize)]
pub struct AutoSSHUser {
    pub prefix: String,
    pub users_count: u64,
    pub exp_date: String,
    pub group: String,
}

#[derive(Deserialize, Serialize)]
pub struct InputSSHUser {
    pub username: String,
    pub password: String,
    pub exp_date: String,
    pub group: String,
    pub shell: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct Claims {
    pub role: String,
    pub exp: u64,
}

#[derive(Debug)]
pub struct Token(String);

#[derive(Debug)]
pub enum ApiTokenError {
    Missing,
    Invalid,
}

impl<'a, 'r> FromRequest<'a, 'r> for Token {
    type Error = ApiTokenError;

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
        let token = request.headers().get_one("Authorization");
        match token {
            Some(token) => Outcome::Success(Token(token.replace("Bearer ", "").to_string())),
            None => Outcome::Failure((Status::Unauthorized, ApiTokenError::Missing)),
        }
    }
}

impl Token {
    pub fn to_string(&self) -> String {
        self.0.clone()
    }

    pub fn validate(&self) -> Result<Role, Json<Value>> {
        let decoded_token = match decode::<Claims>(
            &self.0,
            &DecodingKey::from_secret(JWT_SECRET),
            &Validation::new(Algorithm::HS256),
        ) {
            Ok(token) => token,
            Err(e) => return Err(Json(json!({"Err": e.to_string()}))),
        };

        Ok(Role::from_str(&decoded_token.claims.role))
    }
}

pub enum Role {
    Privileged,
    Normal,
}

impl Role {
    pub fn from_str(role: &str) -> Self {
        match role.to_lowercase().as_str() {
            "privileged" => Role::Privileged,
            _ => Role::Normal,
        }
    }

    pub fn to_string(&self) -> String {
        match &self {
            Role::Privileged => String::from("privileged"),
            _ => String::from("normal"),
        }
    }
}

