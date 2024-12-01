use std::env;
use rocket::{get, launch, routes};

#[macro_use]
extern crate rocket;


#[get("/read")]
fn read_data() -> String {
    let hostname = env::var("HOSTNAME").unwrap_or("default_value".to_string());
    format!("Reading data from: {}", hostname)
}

#[launch]
fn rocket() -> _ {
    println!("Starting Rocket server...");
    rocket::build()
        .mount("/", routes![read_data])
}