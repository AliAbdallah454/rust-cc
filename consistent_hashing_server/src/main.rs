#[macro_use] extern crate rocket;

use std::collections::HashMap;
use std::hash::DefaultHasher;
use std::net::UdpSocket;
use std::sync::Mutex;

use my_consistent_hashing::transaction::{self, Transaction};
use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket::{tokio, State};
use my_consistent_hashing::consistent_hashing::ConsistentHashing;

use aws_config::{meta::region::RegionProviderChain, BehaviorVersion};

use consisten_hashing_server::ecs_functions::launch_task;
use consisten_hashing_server::ecs_functions::stop_task;
use serde_json::Value;

#[derive(Deserialize)]
struct Input {
    value: String,
}

#[derive(Serialize)]
struct GetNodeOutput {
    node: String,
    hash: String,
}

#[derive(Serialize, Deserialize)]
struct TaskInfo {
    pub cluster_name: String,
    pub task_name: String
}

#[post("/add-task", format = "json", data = "<input>")]
async fn add_task(input: Json<TaskInfo>, ecs: &State<aws_sdk_ecs::Client>) -> String {

    let cluster_name = input.cluster_name.clone();
    let task_name = input.task_name.clone();

    println!("cluster_name: {}", cluster_name);
    println!("task_name: {}", task_name);

    let launch_response = launch_task(ecs, &cluster_name, &task_name).await;

    println!("{:?}", launch_response);

    return String::from("Task Launched");

}

#[post("/add-node", format = "json", data = "<input>")]
async fn add_node(input: Json<Input>, ring: &State<Mutex<ConsistentHashing>>) -> Json<Value> {

    let input_value = input.value.clone();
    println!("adding node with ip: {}", input_value);
    let mut ring = ring.lock().expect("Failed to lock the consistent hashing ring");
    let transactions = match ring.add_node(&input_value) {
        Ok(transactions) => transactions,
        Err(e) => {
            println!("{:?}", e);
            return Json(serde_json::json!({
                "status": "error",
                "message": "Failed to add node"
            }));
        }
    };

    drop(ring);

    for t in &transactions {
        println!("{:?}", t);
    }

    let mut groups = HashMap::new();

    for transaction in transactions {
        let key = (transaction.source.clone(), transaction.destination.clone());
        groups
            .entry(key)
            .or_insert_with(Vec::new)
            .push(transaction);
    }

    for (key, transactions) in groups {
        tokio::spawn(async move {

            let source = &key.0;
            let destination = &key.1;

            let client = reqwest::Client::new();

            let source_uri = format!("http://{source}:7000/transactions");
            // let source_uri = format!("http://{source}/transactions");
            let get_response = client.post(&source_uri).json(&transactions).send().await.expect("this should not panic");
            let data = get_response.text().await.unwrap();
            let json_data: Value = serde_json::from_str(&data).unwrap();

            let (exclusive, token): (Value, String) = serde_json::from_value(json_data).unwrap();
            println!("{:?}", exclusive);
            println!("{}", token);
            
            let destination_uri = format!("http://{destination}:7000/exclusive");
            // let destination_uri = format!("http://{destination}/exclusive");
            let post_response = client.post(&destination_uri).json(&exclusive).send().await.expect("this should not panic");
            let post_data = post_response.text().await.unwrap();

            let delete_uri = format!("http://{source}:7000/delete-batch/{}", &token);

            let delete_response = client.post(delete_uri).send().await;

        });

    }
    
    return Json(serde_json::json!({
        "status": "processing"
    }));

}

#[post("/get-node", format = "json", data = "<input>")]
fn get_node(input: Json<Input>, ring: &State<Mutex<ConsistentHashing>>) -> Json<GetNodeOutput> {
    let input_value = input.value.clone();
    println!("Input val is: {}", input_value);
    let ring = ring.lock().expect("Failed to lock the consistent hashing ring");
    let res = ring.get_node(&input_value);
    let node = res.0.unwrap().to_string();
    let hash = res.1.unwrap().to_string(); // parse hash to string

    println!("hash {} in {}", &hash, &node);

    Json(GetNodeOutput {
        node,
        hash,
    })
}

// #[post("/remove-node", format = "json", data = "<input>")]
// fn remove_node(input: Json<Input>, ring: &State<Mutex<ConsistentHashing<DefaultHasher>>>) -> Json<Vec<Transaction>> {
//     let input_value = input.value.clone();
//     println!("Removing node {}", input_value);
//     let mut ring = ring.lock().expect("Failed to lock the consistent hashing ring");
//     let transactions = ring.remove_node(&input_value).unwrap();
//     return Json(transactions);
// }

#[get("/")]
fn hello() -> &'static str {
    "Hello, world!"
}

fn get_private_ip() -> Result<String, Box<dyn std::error::Error>> {
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.connect("8.8.8.8:80")?;
    let local_addr = socket.local_addr()?;
    Ok(local_addr.ip().to_string())
}

#[launch]
async fn rocket() -> _ {

    println!("Private ip: {}", get_private_ip().unwrap());

    let region_provider = RegionProviderChain::default_provider().or_else("eu-west-3");
    let config = aws_config::defaults(BehaviorVersion::latest())
        .region(region_provider)
        .load()
        .await;

    let ecs = aws_sdk_ecs::Client::new(&config);

    println!("Initialized ring");
    let ring = ConsistentHashing::new(5);
    let ring = Mutex::new(ring);

    rocket::build()
        .manage(ring)
        .manage(ecs)
        .mount("/", routes![
            hello,
            get_node,
            add_node,
            add_task
        ])
}