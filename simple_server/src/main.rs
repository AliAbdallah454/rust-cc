use std::env;
use rocket::response::content::RawJson;
use rocket::{get, post, routes, launch};
use rocket::tokio::fs;

use rocket::serde::json::Json;
use serde_json::Value;


#[macro_use] extern crate rocket;

#[get("/")]
fn index() -> &'static str {
    "Welcome to the Rocket API!"
}


#[post("/indexing", format = "json", data = "<json_data>")]
async fn indexing(json_data: Json<Value>) -> &'static str {
    let folder_path = env::var("FOLDER_PATH").expect("FOLDER_PATH must be set");
    println!("Received JSON data: {:?}", json_data);

    return "Json received";
}

// Launch`` the Rocket application
#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![index, indexing])
}