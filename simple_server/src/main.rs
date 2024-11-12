#[macro_use] extern crate rocket;

// GET request handler
#[get("/")]
fn index() -> &'static str {
    "Welcome to the Rocket API!"
}

// Launch the Rocket application
#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![index])
}