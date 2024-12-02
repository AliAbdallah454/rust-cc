#[macro_use] extern crate rocket;

use std::hash::DefaultHasher;
use std::sync::Mutex;

use my_consistent_hashing::transaction::Transaction;
use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket::State;
use my_consistent_hashing::consistent_hashing::ConsistentHashing;

use aws_config::{meta::region::RegionProviderChain, BehaviorVersion};

use consisten_hashing_server::ecs_functions::launch_task;

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
async fn add_node(input: Json<Input>, ring: &State<Mutex<ConsistentHashing<DefaultHasher>>>, ecs: &State<aws_sdk_ecs::Client>) -> Json<Vec<Transaction>> {
    let input_value = input.value.clone();

    let cluster_name = String::from("aa-sdk-cluster");
    let task_name = String::from("writer-task");

    let launch_response = launch_task(ecs, &cluster_name, &task_name).await;

    match launch_response {
        Ok(_) => println!("Worked"),
        Err(e) => println!("{:?}", e)
    }

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
async fn rocket() -> _ {

    let region_provider = RegionProviderChain::default_provider().or_else("eu-west-3");
    let config = aws_config::defaults(BehaviorVersion::latest())
        .region(region_provider)
        .load()
        .await;

    let ecs = aws_sdk_ecs::Client::new(&config);

    println!("Initialized ring");
    println!("This should work now");
    let ring = ConsistentHashing::<DefaultHasher>::new(2);
    let ring = Mutex::new(ring);

    rocket::build()
        .manage(ring)
        .manage(ecs)
        .mount("/", routes![hello, get_node, remove_node, add_node])
}
