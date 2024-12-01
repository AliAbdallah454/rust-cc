#[macro_use] extern crate rocket;

use std::hash::DefaultHasher;
use std::sync::Mutex;

use my_consistent_hashing::transaction::Transaction;
use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket::State;
use my_consistent_hashing::consistent_hashing::ConsistentHashing;

#[derive(Deserialize)]
struct Input {
    value: String,
}

#[derive(Serialize)]
struct Output {
    input: String,
    node: String,
}

#[post("/get-node", format = "json", data = "<input>")]
fn get_node(input: Json<Input>, ring: &State<Mutex<ConsistentHashing<DefaultHasher>>>) -> Json<Output> {
    let input_value = input.value.clone();
    println!("Input val is: {}", input_value);
    let ring = ring.lock().expect("Failed to lock the consistent hashing ring");
    let node = ring.get_node(&input_value).unwrap().to_string();



    Json(Output {
        input: input_value,
        node,
    })
}

#[post("/add-node", format = "json", data = "<input>")]
fn add_node(input: Json<Input>, ring: &State<Mutex<ConsistentHashing<DefaultHasher>>>) -> Json<Vec<Transaction>> {
    let input_value = input.value.clone();
    println!("Adding node {}", input_value);
    let mut ring = ring.lock().expect("Failed to lock the consistent hashing ring");
    let transactions = ring.add_node(&input_value).unwrap();
    return Json(transactions);
}

#[post("/remove-node", format = "json", data = "<input>")]
fn remove_node(input: Json<Input>, ring: &State<Mutex<ConsistentHashing<DefaultHasher>>>) -> Json<Vec<Transaction>> {
    let input_value = input.value.clone();
    println!("Removing node {}", input_value);
    let mut ring = ring.lock().expect("Failed to lock the consistent hashing ring");
    let transactions = ring.remove_node(&input_value).unwrap();
    return Json(transactions);
}

#[get("/")]
fn hello() -> &'static str {
    "Hello, world!"
}

#[launch]
fn rocket() -> _ {
    let mut ring = ConsistentHashing::<DefaultHasher>::new(2);
    // for i in 0..3 {
    //     ring.add_node(format!("127.0.0.1:900{}", i).as_str()).unwrap();
    // }
    println!("Initialized ring");

    let ring = Mutex::new(ring);

    rocket::build()
        .manage(ring)
        .mount("/", routes![hello, get_node, remove_node, add_node])
}
