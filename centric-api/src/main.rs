#[macro_use]
extern crate rocket;

use chrono::{Duration, Local, NaiveDate};
use diesel::query_dsl::methods::*;
use diesel::result::Error as DieselError;
use diesel::ExpressionMethods;
use diesel::RunQueryDsl;
use lib::schema::nodes::dsl::nodes;
use lib::schema::sells::dsl::sells;
use lib::schema::services::dsl::services;
use lib::schema::users::dsl::users;
use lib::{consts, models::*, schema};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use reqwest::Error as ReqwestError;
use rocket::http::Status;
use rocket::response::{status::Created, Debug};
use rocket::serde::json::{json, Json, Value as JsonValue};
use rocket_sync_db_pools::database;
use std::time::SystemTime;

use lib::Db;

macro_rules! api_err {
    ($msg:expr) => {
        json!({ "Err": { "msg": $msg } })
    };
}

type JsonRes<T> = Json<JsonOk<T>>;

fn parse_addr(addr: &str) -> Result<String, (Status, JsonValue)> {
    let node_addr = match url::Url::parse(&addr) {
        Ok(addr) => {
            if !(addr.scheme().to_lowercase() == "http" || addr.scheme().to_lowercase() == "https")
            {
                return Err((
                    Status::UnprocessableEntity,
                    api_err!("internal: node-api's address should be over HTTP or HTTPS protocols"),
                ));
            } else {
                addr
            }
        }
        Err(_e) => {
            return Err((
                Status::UnprocessableEntity,
                api_err!("internal: node-api's address is not a valid URL"),
            ))
        }
    };

    let fmt_addr = format!(
        "{}://{}:{}",
        node_addr.scheme().to_lowercase(),
        node_addr.host().unwrap(),
        node_addr.port().unwrap_or(8010)
    );

    Ok(fmt_addr)
}

#[get("/ping")]
async fn ping() -> &'static str {
    "pong"
}

/* NODES ROUTING */

#[post("/update_node/<node_id>", data = "<node_info>")]
async fn update_node(
    db: Db,
    node_id: i32,
    node_info: Json<UpdateNode>,
) -> Result<Created<JsonRes<Node>>, (Status, JsonValue)> {
    let node_addr = match &node_info.address {
        Some(addr) => Some(parse_addr(&addr)?),
        None => None,
    };

    let node_value = UpdateNode {
        address: node_addr,
        token: node_info.token.clone(),
        status: node_info.status,
    };

    let node_info = Node::update(&db, node_id, node_value).await?;

    Ok(Created::new("/update_node").body(JsonOk::from(node_info)))
}

#[post("/new_node", data = "<node_info>")]
async fn new_node(
    db: Db,
    node_info: Json<NewNode>,
) -> Result<Created<JsonRes<Node>>, (Status, JsonValue)> {
    let node_addr = parse_addr(&node_info.address)?;

    let node_value = NewNode {
        address: node_addr,
        token: node_info.token.clone(),
        status: node_info.status,
    };

    let node_info = Node::insert(&db, node_value).await?;

    Ok(Created::new("/new_node").body(JsonOk::from(node_info)))
}

#[get("/node_info/<node_id>")]
async fn node_info(db: Db, node_id: i32) -> Result<JsonRes<JsonValue>, (Status, JsonValue)> {
    let node_info = Node::find_by_id(&db, node_id).await?;

    Ok(JsonOk::from(node_info.info().await?))
}

#[get("/hw_stats/<node_id>")]
async fn hw_stats(db: Db, node_id: i32) -> Result<JsonRes<JsonValue>, (Status, JsonValue)> {
    let node_info = Node::find_by_id(&db, node_id).await?;

    Ok(JsonOk::from(node_info.hw_stats().await?))
}

#[get("/net_stats/<node_id>")]
async fn net_stats(db: Db, node_id: i32) -> Result<JsonRes<JsonValue>, (Status, JsonValue)> {
    let node_info = Node::find_by_id(&db, node_id).await?;

    Ok(JsonOk::from(node_info.net_stats().await?))
}

#[post("/delete_node/<node_id>")]
async fn delete_node(db: Db, node_id: i32) -> Result<JsonRes<JsonValue>, (Status, JsonValue)> {
    let node_deleted = Node::delete_by_id(&db, node_id).await?;

    if node_deleted {
        Ok(JsonOk::from(
            json!({"node_id": node_id, "status": "deleted"}),
        ))
    } else {
        Ok(JsonOk::from(
            json!({"node_id": node_id, "status": "node doesn't exist"}),
        ))
    }
}

#[get("/get_node/<node_id>")]
async fn get_node(db: Db, node_id: i32) -> Result<JsonRes<Node>, (Status, JsonValue)> {
    let node_info = Node::find_by_id(&db, node_id).await?;

    Ok(JsonOk::from(node_info))
}

#[get("/nodes_list")]
async fn nodes_list(db: Db) -> Result<JsonRes<Vec<Node>>, (Status, JsonValue)> {
    let nodes_list = Node::list(&db).await?;

    Ok(JsonOk::from(nodes_list))
}

/* SERVICES ROUTING */

#[post("/new_service", data = "<service_info>")]
async fn new_service(
    db: Db,
    service_info: Json<NewService>,
) -> Result<Created<JsonRes<Service>>, (Status, JsonValue)> {
    let service_value = service_info.0;

    let service_info = Service::insert(&db, service_value).await?;

    Ok(Created::new("/new_service").body(JsonOk::from(service_info)))
}

#[post("/update_service/<service_id>", data = "<service_info>")]
async fn update_service(
    db: Db,
    service_id: i32,
    service_info: Json<UpdateService>,
) -> Result<Created<JsonRes<Service>>, (Status, JsonValue)> {
    let service_value = service_info.0;

    let service_info = Service::update(&db, service_id, service_value).await?;

    Ok(Created::new("/update_node").body(JsonOk::from(service_info)))
}

#[post("/delete_service/<service_id>")]
async fn delete_service(
    db: Db,
    service_id: i32,
) -> Result<JsonRes<JsonValue>, (Status, JsonValue)> {
    let service_deleted = Service::delete_by_id(&db, service_id).await?;

    if service_deleted {
        Ok(JsonOk::from(
            json!({"service_id": service_id, "status": "deleted"}),
        ))
    } else {
        Ok(JsonOk::from(
            json!({"service_id": service_id, "status": "service doesn't exist"}),
        ))
    }
}

#[get("/services_list")]
async fn services_list(db: Db) -> Result<JsonRes<Vec<Service>>, (Status, JsonValue)> {
    let services_list = Service::list(&db).await?;
    Ok(JsonOk::from(services_list))
}

#[get("/get_service/<service_id>")]
async fn get_service(db: Db, service_id: i32) -> Result<JsonRes<Service>, (Status, JsonValue)> {
    let service_info = Service::find_by_id(&db, service_id).await?;

    Ok(JsonOk::from(service_info))
}

/* USERS ROUTING */

#[post("/new_user", data = "<user_info>")]
async fn new_user(
    db: Db,
    user_info: Json<NewUser>,
) -> Result<Created<JsonRes<User>>, (Status, JsonValue)> {
    let user_exists = User::exists(&db, user_info.id).await?;

    if user_exists {
        return Err((Status::ImATeapot, api_err!("ingore: user exists")));
    }

    let user_value = user_info.0;
    let user_info = User::insert(&db, user_value).await?;

    Ok(Created::new("/new_user").body(JsonOk::from(user_info)))
}

#[get("/user_refs/<user_id>")]
async fn user_refs(db: Db, user_id: i64) -> Result<JsonRes<Vec<User>>, (Status, JsonValue)> {
    let refs = User::refs(&db, user_id).await?;

    Ok(JsonOk::from(refs))
}

#[get("/users_list")]
async fn users_list(db: Db) -> Result<JsonRes<Vec<User>>, (Status, JsonValue)> {
    let users_list = User::list(&db).await?;

    Ok(JsonOk::from(users_list))
}

/* SELLS ROUTING */

#[post("/new_sell", data = "<sell_info>")]
async fn new_sell(
    db: Db,
    sell_info: Json<NewSell>,
) -> Result<Created<JsonRes<Sell>>, (Status, JsonValue)> {
    let user_id = sell_info.user_id;

    let user_info = User::find_by_id(&db, user_id).await?;

    let sell_value = NewSell::new_unverified(
        sell_info.user_id,
        user_info.ref_id,
        sell_info.service_id,
        sell_info.node_id,
    );

    let sell_info = Sell::insert(&db, sell_value).await?;
    Ok(Created::new("/new_sell").body(JsonOk::from(sell_info)))
}

#[post("/verify_sell/<sell_id>", data = "<account_info>")]
async fn verify_sell(
    db: Db,
    sell_id: i32,
    account_info: Json<AccountInfo>,
) -> Result<JsonRes<Sell>, (Status, JsonValue)> {
    let sell_info = Sell::find_by_id(&db, sell_id).await?;

    let service_info = Service::find_by_id(&db, sell_info.service_id).await?;

    let node_info = Node::find_by_id(&db, sell_info.node_id).await?;

    let sshuser_info = node_info
        .useradd(sell_id, &service_info, &account_info.0)
        .await?;

    let update_sell = UpdateSell::verify(sshuser_info);

    let new_sell_info = Sell::update(&db, sell_id, update_sell).await?;

    Ok(JsonOk::from(new_sell_info))
    // WIP
}

#[get("/sell_info/<sell_id>")]
async fn sell_info(db: Db, sell_id: i32) -> Result<JsonRes<Sell>, (Status, JsonValue)> {
    let sell_info = Sell::find_by_id(&db, sell_id).await?;
    Ok(JsonOk::from(sell_info))
}

#[get("/sells_list")]
async fn sells_list(db: Db) -> Result<JsonRes<Vec<Sell>>, (Status, JsonValue)> {
    let sells_list = Sell::list(&db).await?;
    Ok(JsonOk::from(sells_list))
}

#[get("/sells_list_by_ref/<ref_id>")]
async fn sells_list_by_ref(db: Db, ref_id: i64) -> Result<JsonRes<Vec<Sell>>, (Status, JsonValue)> {
    let sells_list = Sell::list_by_ref(&db, ref_id).await?;
    Ok(JsonOk::from(sells_list))
}

#[get("/sells_list_by_user/<user_id>")]
async fn sells_list_by_user(
    db: Db,
    user_id: i64,
) -> Result<JsonRes<Vec<Sell>>, (Status, JsonValue)> {
    let sells_list = Sell::list_by_user(&db, user_id).await?;
    Ok(JsonOk::from(sells_list))
}

// AUTH SECTION:
//
// WIP

#[post("/register", data = "<login_info>")]
async fn register(
    db: Db,
    login_info: Json<LoginInfo>,
) -> Result<Created<JsonRes<String>>, (Status, JsonValue)> {
    if let None = login_info.0.admin_key {
        return Err((Status::BadRequest, api_err!("Admin_Key Missing!")));
    }

    if Some(consts::ADMIN_KEY.to_string()) != login_info.0.admin_key {
        return Err((Status::BadRequest, api_err!("Invalid admin_key")));
    }

    let password = login_info.0.password;
    let username = login_info.0.username;

    if Login::exists(&db, username.to_owned()).await? {
        return Err((Status::BadRequest, api_err!("user already exist")));
    }

    if !lib::is_valid_username(&username) {
        return Err((Status::BadRequest, api_err!("invalid username")));
    }

    let password_hash = lib::hash_password(&password);

    let login_value = NewLogin {
        username,
        password_hash,
        admin: None,
    };

    Login::insert(&db, login_value).await?;

    Ok(Created::new("/register").body(JsonOk::from(format!("user sucessfully created!"))))
}

#[post("/login", data = "<login_info>")]
async fn login(
    db: Db,
    login_info: Json<LoginInfo>,
) -> Result<Created<JsonRes<String>>, (Status, JsonValue)> {
    let password = login_info.0.password;
    let username = login_info.0.username;

    if !Login::exists(&db, username.to_owned()).await? {
        return Err((Status::BadRequest, api_err!("user doesn't exist")));
    }

    let user_info = Login::find_by_username(&db, username).await?;

    if !lib::verify_hash(&password, &user_info.password_hash) {
        return Err((Status::BadRequest, api_err!("password is wrong")));
    }

    let token = lib::token::Token::generate();

    Ok(Created::new("/login").body(JsonOk::from(token)))
}

#[post("/verify_token")]
async fn verify_token(token: lib::token::Token) -> Result<(), (Status, JsonValue)> {
    token.validate()?;
    Ok(())
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(Db::fairing())
        .mount(
            "/nodes",
            routes![
                ping,
                nodes_list,
                get_node,
                delete_node,
                update_node,
                node_info,
                new_node,
                hw_stats,
                net_stats,
            ],
        )
        .mount(
            "/services",
            routes![
                new_service,
                update_service,
                delete_service,
                services_list,
                get_service
            ],
        )
        .mount("/users", routes![new_user, users_list, user_refs])
        .mount(
            "/sells",
            routes![
                new_sell,
                verify_sell,
                sell_info,
                sells_list,
                sells_list_by_ref,
                sells_list_by_user
            ],
        )
        .mount("/auth", routes![register, login, verify_token])
}

