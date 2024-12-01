use rocket::{post, serde::json::Json};
use serde_json::Value;
use std::env;

#[macro_use]
extern crate rocket;

#[post("/", data = "<write_req>")]
fn write_data(write_req: Json<Value>) -> String {

    let hostname = env::var("HOSTNAME").unwrap_or("default_value".to_string());

    println!("obj: {:?}", write_req);
    format!("Data written successfully to {}", hostname)
}

#[launch]
fn rocket() -> _ {
    println!("Starting Rocket server...");
    rocket::build()
        .mount("/", routes![write_data])
}