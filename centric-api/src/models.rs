use crate::consts;
use crate::node_models;
use crate::schema::*;
use crate::Db;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::prelude::*;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use reqwest::Error as ReqwestError;
use rocket::http::Status;
use rocket::serde::{Deserialize, Serialize};
use std::cmp::{Eq, Ord, PartialEq, PartialOrd};
use std::time::SystemTime;

use crate::schema::nodes::{dsl::nodes as nodes_dsl, table as nodes_table};
use crate::schema::sells::{
    dsl::sells as sells_dsl, ref_id as sells_ref_id, table as sells_table, user_id as sells_user_id,
};
use crate::schema::services::{dsl::services as services_dsl, table as services_table};
use crate::schema::users::{dsl::users as users_dsl, ref_id as field_ref_id, table as users_table};

use crate::schema::logins::{
    dsl::logins as logins_dsl, table as logins_table, username as login_username,
};

use diesel::result::Error as DieselError;
use rocket::serde::json::{json, Json, Value as JsonValue};

pub type ApiError = (Status, JsonValue);

#[derive(Serialize, Deserialize)]
pub struct JsonOk<T> {
    pub Ok: T,
}

impl<T> JsonOk<T> {
    pub fn from(value: T) -> Json<Self> {
        Json(JsonOk { Ok: value })
    }
}

trait JsonResponseError {
    fn jsonify(&self) -> ApiError;
}

impl JsonResponseError for DieselError {
    fn jsonify(&self) -> ApiError {
        match self {
            DieselError::NotFound => (
                Status::InternalServerError,
                json!({
                "Err": {
                    "type": "db",
                    "code": 404,
                    "msg": "Record not found",
                    "raw_msg": format!("{}", self)
                }}),
            ),

            _ => (
                Status::InternalServerError,
                json!({
                "Err": {
                    "type": "db",
                    "code": 500,
                    "msg": "Unexpected Error",
                    "raw_msg": format!("{}", self)
                }}),
            ),
        }
    }
}

impl JsonResponseError for ReqwestError {
    fn jsonify(&self) -> ApiError {
        if self.is_connect() {
            (
                Status::InternalServerError,
                json!({
                    "Err": {
                        "type": "req",
                        "code": 111,
                        "msg": "connection refused",
                        "raw_msg": format!("{}", self)
                    }
                }),
            )
        } else if self.is_timeout() {
            (
                Status::InternalServerError,
                json!({
                    "Err": {
                        "type": "req",
                        "code": 110,
                        "msg": "connection timeout",
                        "raw_msg": format!("{}", self)
                    }
                }),
            )
        } else if self.is_request() {
            (
                Status::InternalServerError,
                json!({
                    "Err": {
                        "type": "req",
                        "code": 110, //fix
                        "msg": "request error",
                        "raw_msg": format!("{}", self)
                    }
                }),
            )
        } else {
            (
                Status::InternalServerError,
                json!({
                    "Err": {
                        "type": "req",
                        "code": 900, //fix
                        "msg": "unexpected error",
                        "raw_msg": format!("{}", self)
                    }
                }),
            )
        }
    }
}

// NODES TABLE
/// Structure representing a node in the system
#[derive(Queryable, Serialize, Deserialize, Ord, Eq, PartialEq, PartialOrd)]
#[diesel(table_name = nodes)]
pub struct Node {
    pub id: i32,
    pub address: String,
    pub token: String,
    pub status: i32,
}

impl Node {
    /// Asynchronously finds a node by its ID
    pub async fn find_by_id(db: &Db, id: i32) -> Result<Node, ApiError> {
        let node_info = db
            .run(move |conn| {
                nodes_dsl
                    .find(id)
                    .get_result::<Node>(conn)
                    .map_err(|err| err.jsonify())
            })
            .await?;

        Ok(node_info)
    }

    /// Asynchronously inserts a new node into the database
    pub async fn insert(db: &Db, node_value: NewNode) -> Result<Node, ApiError> {
        let node_info = db
            .run(move |conn| {
                diesel::insert_into(nodes_table)
                    .values(node_value)
                    .get_result(conn)
                    .map_err(|err| err.jsonify())
            })
            .await?;

        Ok(node_info)
    }

    /// Asynchronously updates an existing node in the database
    pub async fn update(
        db: &Db,
        id: i32,
        node_value: UpdateNode,
    ) -> Result<Node, (Status, JsonValue)> {
        let node_info = db
            .run(move |conn| {
                diesel::update(nodes_dsl.find(id))
                    .set(node_value)
                    .get_result::<Node>(conn)
                    .map_err(|err| err.jsonify())
            })
            .await?;

        Ok(node_info)
    }

    /// Asynchronously retrieves a list of all nodes from the database
    pub async fn list(db: &Db) -> Result<Vec<Node>, ApiError> {
        let nodes_list = db
            .run(|conn| nodes_table.load(conn).map_err(|err| err.jsonify()))
            .await?;

        Ok(nodes_list)
    }

    /// Asynchronously deletes a node by its ID
    pub async fn delete_by_id(db: &Db, id: i32) -> Result<bool, ApiError> {
        let deleted_nodes = db
            .run(move |conn| {
                diesel::delete(nodes_dsl.find(id))
                    .execute(conn)
                    .map_err(|err| err.jsonify())
            })
            .await?;

        if deleted_nodes > 0 {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    // REQWEST SECTION

    /// Asynchronously fetches hardware statistics from the node
    pub async fn hw_stats(&self) -> Result<JsonValue, ApiError> {
        let client = reqwest::Client::new();

        let request_builder = client
            .get(format!("{}{}", &self.address, consts::HW_STATS_PATH))
            .header(
                AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {}", &self.token)).unwrap(),
            );

        let node_api_response = request_builder.send().await.map_err(|err| err.jsonify())?;
        Ok(node_api_response
            .json()
            .await
            .map_err(|err| err.jsonify())?)
    }

    /// Asynchronously fetches network statistics from the node
    pub async fn net_stats(&self) -> Result<JsonValue, ApiError> {
        let client = reqwest::Client::new();

        let request_builder = client
            .get(format!("{}{}", &self.address, consts::NET_STATS_PATH))
            .header(
                AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {}", &self.token)).unwrap(),
            );

        let node_api_response = request_builder.send().await.map_err(|err| err.jsonify())?;
        Ok(node_api_response
            .json()
            .await
            .map_err(|err| err.jsonify())?)
    }

    /// Asynchronously retrieves information about the node
    pub async fn info(&self) -> Result<JsonValue, ApiError> {
        let node_api_response =
            reqwest::get(format!("{}{}", &self.address, consts::NODE_INFO_PATH))
                .await
                .map_err(|err| err.jsonify())?;

        Ok(node_api_response
            .json()
            .await
            .map_err(|err| err.jsonify())?)
    }

    /// Asynchronously changes the password for a user on the node
    pub async fn change_pass(
        &self,
        user_id: i32,
        service_info: &Service,
        account_info: &AccountInfo,
    ) -> Result<node_models::SSHUser, ApiError> {
        let sshuser_json = node_models::InputSSHUser::auto_gen(
            service_info.max_logins,
            user_id,
            account_info.days,
        );

        let client = reqwest::Client::new();
        let request_builder = client
            .post(format!("{}{}", &self.address, consts::USERADD_PATH))
            .header(
                AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {}", &self.token)).unwrap(),
            );

        let node_api_response = request_builder
            .json(&sshuser_json)
            .send()
            .await
            .map_err(|err| err.jsonify())?;

        let sshuser_info = node_api_response
            .json::<Result<node_models::SSHUser, JsonValue>>()
            .await
            .map_err(|err| err.jsonify())?;

        sshuser_info.map_err(|err| (Status::InternalServerError, err))
    }

    /// Asynchronously adds a new user to the node
    pub async fn useradd(
        &self,
        user_id: i32,
        service_info: &Service,
        account_info: &AccountInfo,
    ) -> Result<node_models::SSHUser, ApiError> {
        let sshuser_json = node_models::InputSSHUser::auto_gen(
            service_info.max_logins,
            user_id,
            account_info.days,
        );

        let client = reqwest::Client::new();
        let request_builder = client
            .post(format!("{}{}", &self.address, consts::USERADD_PATH))
            .header(
                AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {}", &self.token)).unwrap(),
            );

        let node_api_response = request_builder
            .json(&sshuser_json)
            .send()
            .await
            .map_err(|err| err.jsonify())?;

        let sshuser_info = node_api_response
            .json::<Result<node_models::SSHUser, JsonValue>>()
            .await
            .map_err(|err| err.jsonify())?;

        sshuser_info.map_err(|err| (Status::InternalServerError, err))
    }
}

/// Structure representing information for creating a new node
#[derive(Insertable, Serialize, Deserialize, Clone)]
#[serde(crate = "rocket::serde")]
#[diesel(table_name = nodes)]
pub struct NewNode {
    pub address: String,
    pub token: String,
    pub status: i32,
}

/// Structure representing information for updating a node
#[derive(Insertable, Serialize, Deserialize, Clone, AsChangeset)]
#[serde(crate = "rocket::serde")]
#[diesel(table_name = nodes)]
pub struct UpdateNode {
    pub address: Option<String>,
    pub token: Option<String>,
    pub status: Option<i32>,
}

// SERVICES TABLE

/// Structure representing a service
#[derive(Queryable, Serialize, Deserialize, Ord, Eq, PartialEq, PartialOrd)]
#[diesel(table_name = services)]
pub struct Service {
    pub id: i32,
    pub max_logins: i32,
    pub max_traffic: Option<i32>,
    pub price: i32,
    pub available: bool,
}

/// Implementation of operations related to services
impl Service {
    /// Asynchronously finds a service by its ID
    pub async fn find_by_id(db: &Db, id: i32) -> Result<Service, ApiError> {
        let service_info = db
            .run(move |conn| {
                services_dsl
                    .find(id)
                    .get_result::<Service>(conn)
                    .map_err(|err| err.jsonify())
            })
            .await?;

        Ok(service_info)
    }

    /// Asynchronously inserts a new service into the database
    pub async fn insert(db: &Db, service_value: NewService) -> Result<Service, ApiError> {
        let service_info = db
            .run(move |conn| {
                diesel::insert_into(services_table)
                    .values(service_value)
                    .get_result(conn)
                    .map_err(|err| err.jsonify())
            })
            .await?;

        Ok(service_info)
    }

    /// Asynchronously updates an existing service in the database
    pub async fn update(
        db: &Db,
        id: i32,
        service_value: UpdateService,
    ) -> Result<Service, ApiError> {
        let service_info = db
            .run(move |conn| {
                diesel::update(services_dsl.find(id))
                    .set(service_value)
                    .get_result::<Service>(conn)
                    .map_err(|err| err.jsonify())
            })
            .await?;

        Ok(service_info)
    }

    /// Asynchronously retrieves a list of all services from the database
    pub async fn list(db: &Db) -> Result<Vec<Service>, ApiError> {
        let services_list = db
            .run(|conn| services_table.load(conn).map_err(|err| err.jsonify()))
            .await?;

        Ok(services_list)
    }

    /// Asynchronously deletes a service by its ID
    pub async fn delete_by_id(db: &Db, id: i32) -> Result<bool, ApiError> {
        let deleted_services = db
            .run(move |conn| {
                diesel::delete(services_dsl.find(id))
                    .execute(conn)
                    .map_err(|err| err.jsonify())
            })
            .await?;

        if deleted_services > 0 {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

/// Structure representing information for creating a new service
#[derive(Insertable, Serialize, Deserialize, Clone)]
#[serde(crate = "rocket::serde")]
#[diesel(table_name = services)]
pub struct NewService {
    pub max_logins: i32,
    pub max_traffic: Option<i32>,
    pub price: i32,
    pub available: Option<bool>,
}

/// Structure representing information for updating a service
#[derive(Insertable, Serialize, Deserialize, Clone, AsChangeset)]
#[serde(crate = "rocket::serde")]
#[diesel(table_name = services)]
pub struct UpdateService {
    pub max_logins: Option<i32>,
    pub max_traffic: Option<i32>,
    pub price: Option<i32>,
    pub available: Option<bool>,
}

// USERS TABLE
/// Structure representing a user in the system
#[derive(Queryable, Serialize, Deserialize, Ord, Eq, PartialEq, PartialOrd)]
#[diesel(table_name = users)]
pub struct User {
    pub id: i64,
    pub ref_id: Option<i64>,
    pub register_date: SystemTime,
}

/// Implementation of operations related to users in the system
impl User {
    /// Asynchronously finds a user by its ID
    pub async fn find_by_id(db: &Db, id: i64) -> Result<User, ApiError> {
        let user_info = db
            .run(move |conn| {
                users_dsl
                    .find(id)
                    .get_result::<User>(conn)
                    .map_err(|err| err.jsonify())
            })
            .await?;

        Ok(user_info)
    }

    /// Asynchronously checks if a user with the given ID exists
    pub async fn exists(db: &Db, id: i64) -> Result<bool, ApiError> {
        let users_count = db
            .run(move |conn| {
                users_dsl
                    .find(id)
                    .execute(conn)
                    .map_err(|err| err.jsonify())
            })
            .await?;

        if users_count != 0 {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Asynchronously inserts a new user into the database
    pub async fn insert(db: &Db, user_value: NewUser) -> Result<User, ApiError> {
        let user_info = db
            .run(move |conn| {
                diesel::insert_into(users_table)
                    .values(user_value)
                    .get_result(conn)
                    .map_err(|err| err.jsonify())
            })
            .await?;

        Ok(user_info)
    }

    /// Asynchronously retrieves a list of all users' referrals from the database
    pub async fn refs(db: &Db, id: i64) -> Result<Vec<User>, ApiError> {
        let refs = db
            .run(move |conn| {
                users_dsl
                    .filter(field_ref_id.eq(id))
                    .load::<User>(conn)
                    .map_err(|err| err.jsonify())
            })
            .await?;

        Ok(refs)
    }

    /// Asynchronously retrieves a list of all users from the database
    pub async fn list(db: &Db) -> Result<Vec<User>, ApiError> {
        let users_list = db
            .run(|conn| users_table.load(conn).map_err(|err| err.jsonify()))
            .await?;

        Ok(users_list)
    }
}

/// Structure representing information for creating a new user
#[derive(Insertable, Serialize, Deserialize, Clone)]
#[serde(crate = "rocket::serde")]
#[diesel(table_name = users)]
pub struct NewUser {
    pub id: i64,
    pub ref_id: Option<i64>,
    pub register_date: Option<SystemTime>,
}

// SELLS TABLE

/// Structure representing a sell record in the system
#[derive(Queryable, Serialize, Deserialize, Ord, Eq, PartialEq, PartialOrd)]
#[diesel(table_name = sells)]
pub struct Sell {
    pub id: i32,
    pub user_id: i64,
    pub ref_id: Option<i64>,
    pub service_id: i32,
    pub node_id: i32,
    pub firstbuy_date: Option<SystemTime>,
    pub invoice_date: Option<SystemTime>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub password_hash: Option<String>,
    pub status: i32,
}

/// Implementation of operations related to sells in the system
impl Sell {
    /// Asynchronously finds a sell by its ID
    pub async fn find_by_id(db: &Db, id: i32) -> Result<Sell, ApiError> {
        let sell_info = db
            .run(move |conn| {
                sells_dsl
                    .find(id)
                    .get_result::<Sell>(conn)
                    .map_err(|err| err.jsonify())
            })
            .await?;

        Ok(sell_info)
    }

    /// Asynchronously inserts a new sell into the database
    pub async fn insert(db: &Db, sell_value: NewSell) -> Result<Sell, ApiError> {
        let sell_info = db
            .run(move |conn| {
                diesel::insert_into(sells_table)
                    .values(sell_value)
                    .get_result(conn)
                    .map_err(|err| err.jsonify())
            })
            .await?;

        Ok(sell_info)
    }

    /// Asynchronously updates an existing sell in the database
    pub async fn update(db: &Db, id: i32, sell_value: UpdateSell) -> Result<Sell, ApiError> {
        let sell_info = db
            .run(move |conn| {
                diesel::update(sells_dsl.find(id))
                    .set(sell_value)
                    .get_result::<Sell>(conn)
                    .map_err(|err| err.jsonify())
            })
            .await?;

        Ok(sell_info)
    }

    /// Asynchronously retrieves a list of all sells from the database
    pub async fn list(db: &Db) -> Result<Vec<Sell>, ApiError> {
        let sells_list = db
            .run(|conn| sells_table.load(conn).map_err(|err| err.jsonify()))
            .await?;

        Ok(sells_list)
    }

    /// Asynchronously retrieves a list of sells by a given referral ID
    pub async fn list_by_ref(db: &Db, ref_id: i64) -> Result<Vec<Sell>, ApiError> {
        let sells_list = db
            .run(move |conn| {
                sells_table
                    .filter(sells_ref_id.eq(ref_id))
                    .load(conn)
                    .map_err(|err| err.jsonify())
            })
            .await?;

        Ok(sells_list)
    }

    /// Asynchronously retrieves a list of sells by a given user ID
    pub async fn list_by_user(db: &Db, user_id: i64) -> Result<Vec<Sell>, ApiError> {
        let sells_list = db
            .run(move |conn| {
                sells_table
                    .filter(sells_user_id.eq(user_id))
                    .load(conn)
                    .map_err(|err| err.jsonify())
            })
            .await?;

        Ok(sells_list)
    }
}

/// Structure representing information for creating a new sell record
#[derive(Insertable, Serialize, Deserialize, Clone)]
#[serde(crate = "rocket::serde")]
#[diesel(table_name = sells)]
pub struct NewSell {
    pub user_id: i64,
    pub ref_id: Option<i64>,
    pub service_id: i32,
    pub node_id: i32,
    pub firstbuy_date: Option<SystemTime>,
    pub invoice_date: Option<SystemTime>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub password_hash: Option<String>,
    pub status: Option<i32>,
}

impl NewSell {
    pub fn new_unverified(
        user_id: i64,
        ref_id: Option<i64>,
        service_id: i32,
        node_id: i32,
    ) -> NewSell {
        NewSell {
            user_id,
            service_id,
            node_id,
            ref_id,
            firstbuy_date: None,
            invoice_date: None,
            username: None,
            password: None,
            password_hash: None,
            status: Some(1),
        }
    }
}

/// Structure representing information for updating a sell record
#[derive(Insertable, Serialize, Deserialize, Clone, AsChangeset)]
#[serde(crate = "rocket::serde")]
#[diesel(table_name = sells)]
pub struct UpdateSell {
    pub user_id: Option<i64>,
    pub ref_id: Option<i64>,
    pub service_id: Option<i32>,
    pub node_id: Option<i32>,
    pub firstbuy_date: Option<SystemTime>,
    pub invoice_date: Option<SystemTime>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub password_hash: Option<String>,
    pub status: Option<i32>,
}

impl UpdateSell {
    pub fn verify(sshuser: node_models::SSHUser) -> UpdateSell {
        let now = SystemTime::now();
        let invoice_date = now + std::time::Duration::from_secs(30 * 24 * 60 * 60);

        UpdateSell {
            user_id: None,
            service_id: None,
            node_id: None,
            ref_id: None,
            firstbuy_date: Some(now),
            invoice_date: Some(invoice_date),
            username: Some(sshuser.username),
            password: None,
            password_hash: Some(sshuser.password_hash),
            status: Some(0),
        }
    }
}

/// Structure representing login information
#[derive(Queryable, Serialize, Deserialize, Ord, Eq, PartialEq, PartialOrd)]
#[diesel(table_name = logins)]
pub struct Login {
    pub id: i32,
    pub username: String,
    pub password_hash: String,
    pub admin: bool,
    pub register_date: SystemTime,
}

/// Implementation of operations related to logins in the system
impl Login {
    /// Asynchronously finds a login by its ID
    pub async fn find_by_id(db: &Db, id: i32) -> Result<Login, ApiError> {
        let login_info = db
            .run(move |conn| {
                logins_dsl
                    .find(id)
                    .get_result::<Login>(conn)
                    .map_err(|err| err.jsonify())
            })
            .await?;

        Ok(login_info)
    }

    /// Asynchronously finds a login by its username
    pub async fn find_by_username(db: &Db, username: String) -> Result<Login, ApiError> {
        let login_info = db
            .run(move |conn| {
                logins_dsl
                    .filter(login_username.eq(&username))
                    .get_result::<Login>(conn)
                    .map_err(|err| err.jsonify())
            })
            .await?;

        Ok(login_info)
    }

    /// Asynchronously inserts a new login into the database
    pub async fn insert(db: &Db, login_value: NewLogin) -> Result<Login, ApiError> {
        let login_info = db
            .run(move |conn| {
                diesel::insert_into(logins_table)
                    .values(login_value)
                    .get_result(conn)
                    .map_err(|err| err.jsonify())
            })
            .await?;

        Ok(login_info)
    }

    /// Asynchronously updates an existing login in the database
    pub async fn update(db: &Db, id: i32, login_value: UpdateLogin) -> Result<Login, ApiError> {
        let login_info = db
            .run(move |conn| {
                diesel::update(logins_dsl.find(id))
                    .set(login_value)
                    .get_result::<Login>(conn)
                    .map_err(|err| err.jsonify())
            })
            .await?;

        Ok(login_info)
    }

    /// Asynchronously retrieves a list of all logins from the database
    pub async fn list(db: &Db) -> Result<Vec<Login>, ApiError> {
        let logins_list = db
            .run(|conn| logins_table.load(conn).map_err(|err| err.jsonify()))
            .await?;

        Ok(logins_list)
    }

    /// Asynchronously checks if a login with the given username exists
    pub async fn exists(db: &Db, username: String) -> Result<bool, ApiError> {
        let users_count = db
            .run(move |conn| {
                logins_dsl
                    .filter(login_username.eq(&username))
                    .execute(conn)
                    .map_err(|err| err.jsonify())
            })
            .await?;

        if users_count != 0 {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

/// Structure representing information for creating a new login
#[derive(Insertable, Serialize, Deserialize, Clone)]
#[serde(crate = "rocket::serde")]
#[diesel(table_name = logins)]
pub struct NewLogin {
    pub username: String,
    pub password_hash: String,
    pub admin: Option<bool>,
}

/// Structure representing information for updating a login
#[derive(Insertable, Serialize, Deserialize, Clone, AsChangeset)]
#[serde(crate = "rocket::serde")]
#[diesel(table_name = logins)]
pub struct UpdateLogin {
    pub username: Option<String>,
    pub password_hash: Option<String>,
    pub admin: Option<bool>,
}

/// Structure representing account information
#[derive(Serialize, Deserialize)]
pub struct AccountInfo {
    pub days: Option<i64>,
}

/// Structure representing login information
#[derive(Serialize, Deserialize)]
pub struct LoginInfo {
    pub admin_key: Option<String>,
    pub username: String,
    pub password: String,
}
