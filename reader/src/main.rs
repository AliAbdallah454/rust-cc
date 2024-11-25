use rocket::{get, launch, routes};
use hostname::get;

#[macro_use]
extern crate rocket;


#[get("/read")]
fn read_data() -> String {
    let hostname = get().unwrap().to_str().unwrap().to_string();
    format!("Reading data from: {}", hostname)
}

#[launch]
fn rocket() -> _ {
    println!("Starting Rocket server...");
    rocket::build()
        .mount("/", routes![read_data])
}