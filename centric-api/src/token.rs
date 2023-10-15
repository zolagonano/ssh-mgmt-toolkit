use jsonwebtoken::{decode,encode, Algorithm, DecodingKey, Validation, EncodingKey, Header};
use rocket::http::Status;
use rocket::request::Outcome;
use rocket::request::{self, FromRequest, Request};
use rocket::serde::json::{json, Json, Value as JsonValue};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::consts::JWT_SECRET;

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

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Token {
    type Error = ApiTokenError;

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let token = request.headers().get_one("Authorization");
        match token {
            Some(token) => Outcome::Success(Token(token.replace("Bearer ", "").to_string())),
            None => Outcome::Failure((Status::Unauthorized, ApiTokenError::Missing)),
        }
    }
}

impl Token {
    pub fn generate() -> String {
        let now = SystemTime::now();
        let unix_time = now.duration_since(UNIX_EPOCH).unwrap().as_secs();

        let claims = Claims {
            role: "normal".to_string(),
            exp: unix_time + (((60*60)*24)*14 /*2 week*/),
        };
        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(JWT_SECRET),
        )
        .unwrap()
    }

    pub fn to_string(&self) -> String {
        self.0.to_string()
    }

    pub fn validate(&self) -> Result<Role, (Status, JsonValue)> {
        let decoded_token = match decode::<Claims>(
            &self.0,
            &DecodingKey::from_secret(JWT_SECRET),
            &Validation::new(Algorithm::HS256),
        ) {
            Ok(token) => token,
            Err(e) => return Err((Status::Unauthorized, json!({"Err": e.to_string()}))),
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
