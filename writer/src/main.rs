use rocket::{post, routes, serde::json::Json, State};
use serde::Deserialize;
use std::sync::Mutex;
use std::{env, path::Path, fs, thread, time::Duration};

#[macro_use]
extern crate rocket;

#[derive(Deserialize)]
struct WriteRequest {
    key: String,
    value: String,
}

#[post("/write", data = "<write_req>")]
fn write_data(write_req: Json<WriteRequest>) -> &'static str {
    println!("Writing data to DB");
    println!("Key: {}", write_req.key);
    "Data written successfully"
}

#[launch]
fn rocket() -> _ {
    println!("Starting Rocket server...");
    rocket::build()
        .mount("/", routes![write_data])
}