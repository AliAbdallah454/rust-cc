#[macro_use] extern crate rocket;

use std::collections::HashMap;
use std::net::UdpSocket;
use std::sync::{Arc, Mutex};

use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket::{tokio, State};
use consistent_hashing_aa::consistent_hashing::ConsistentHashing;

use aws_config::{meta::region::RegionProviderChain, BehaviorVersion};

use consisten_hashing_server::ecs_functions::{get_ecs_task_private_ips, launch_task};
// use consisten_hashing_server::ecs_functions::stop_task;
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

#[post("/remove-task/<ip>")]
async fn remove_task(ip: &str, ring: &State<Arc<Mutex<ConsistentHashing>>>, _ecs: &State<aws_sdk_ecs::Client>) -> Json<Value> {

    let ring_ref = Arc::clone(ring);
    let transactions = {
        let mut ring = ring_ref.lock().unwrap();
        if !ring.nodes.contains(ip) {
            return Json(serde_json::json!({
                "message": "This node doesn't exists"
            }));
        }
        let transactions = ring.remove_node(ip).unwrap();
        drop(ring);
        transactions
    };

    if transactions.len() == 0 {
        return Json(serde_json::json!({
            "message": "No transactions found"
        }));
    }

    let mut groups = HashMap::new();

    for transaction in transactions {
        let key = (transaction.source.clone(), transaction.destination.clone());
        groups
            .entry(key)
            .or_insert_with(Vec::new)
            .push(transaction);
    }
    let mut handles = vec![];

    for (key, transactions) in groups {
        let handle = tokio::spawn(async move {

            println!("Current transactions: ");
            for transaction in &transactions {
                println!("{:?}", transaction);
            }

            let source = &key.0;
            let destination = &key.1;

            let client = reqwest::Client::new();

            // get exclusive from source
            let source_uri = format!("http://{source}:7000/transactions");
            let get_response = client.post(&source_uri).json(&transactions).send().await.expect("this should not panic");

            if get_response.status() != 200 {
                println!("Unable to get exclusive");
                return;
            }

            let data = get_response.text().await.unwrap();
            let json_data: Value = serde_json::from_str(&data).unwrap();
            let (exclusive, _): (Value, String) = serde_json::from_value(json_data).unwrap();

            println!("{:?}", exclusive);
            
            // send exclusive to destination
            let destination_uri = format!("http://{destination}:7000/exclusive");
            let mut post_response = None;
            for _ in 0..2 {
                post_response = Some(client.post(&destination_uri).json(&exclusive).send().await);
                if post_response.as_ref().unwrap().is_ok() {
                    break;
                }
                println!("Connection refused for {}, retrying", &destination);
                tokio::time::sleep(tokio::time::Duration::from_millis(250)).await;
            }

            let post_response = post_response.unwrap().unwrap();
            if post_response.status() != 200 {
                println!("Exclusive wasn't ingested");
                return;
            }
            println!("{:?}", post_response);

            let post_data = post_response.text().await.unwrap();
            println!("data: {}", post_data);

        });
        handles.push(handle);
    }

    for handle in handles {
        match handle.await {
            Ok(_) => {
                // let cluster_name = String::from("aa-sdk-cluster");
                // stop_task(ecs, &cluster_name, ip).await.unwrap();
            },
            Err(e) => {
                println!("A handle caused an error: {:?}", e);
                return Json(serde_json::json!({
                    "message": "An error has occured,"
                }));
            }
        }
    }

    println!("All transactions finished");

    return Json(serde_json::json!({
        "message": "Transactions finished"
    }));

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
async fn add_node(input: Json<Input>, ring: &State<Arc<Mutex<ConsistentHashing>>>) -> Json<Value> {

    let input_value = input.value.clone();
    println!("Adding node with ip: {}", input_value);
    let ring_ref = Arc::clone(ring);
    let mut ring = ring_ref.lock().expect("Failed to lock the consistent hashing ring");
    
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

    println!("Transactions to be made: ");
    for transaction in &transactions {
        println!("{:?}", transaction);
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

            println!("Current transactions: ");
            for transaction in &transactions {
                println!("{:?}", transaction);
            }

            let source = &key.0;
            let destination = &key.1;

            let client = reqwest::Client::new();

            let source_uri = format!("http://{source}:7000/transactions");
            let get_response = client.post(&source_uri).json(&transactions).send().await.expect("this should not panic");

            if get_response.status() != 200 {
                println!("Unable to get exclusive");
                return;
            }

            let data = get_response.text().await.unwrap();
            let json_data: Value = serde_json::from_str(&data).unwrap();
            let (exclusive, token): (Value, String) = serde_json::from_value(json_data).unwrap();

            println!("{:?}", exclusive);
            
            let destination_uri = format!("http://{destination}:7000/exclusive");
            let mut post_response = None;
            for _ in 0..2 {
                post_response = Some(client.post(&destination_uri).json(&exclusive).send().await);
                if post_response.as_ref().unwrap().is_ok() {
                    break;
                }
                println!("Connection refused for {}, retrying", &destination);
                tokio::time::sleep(tokio::time::Duration::from_millis(250)).await;
            }

            let post_response = post_response.unwrap().unwrap();
            if post_response.status() != 200 {
                println!("Exclusive wasn't ingested");
                return
            }
            println!("{:?}", post_response);

            let post_data = post_response.text().await.unwrap();
            println!("data: {}", post_data);

            let delete_uri = format!("http://{source}:7000/delete-batch/{}", &token);
            let delete_response = client.post(delete_uri).send().await.unwrap();

            if delete_response.status() != 200 {
                println!("Data may have not been delete");
                return;
            }

            let delete_data = delete_response.text().await.unwrap();
            println!("{}", delete_data);

        });
    }
    
    return Json(serde_json::json!({
        "status": "processing"
    }));

}

#[post("/get-node", format = "json", data = "<input>")]
async fn get_node(input: Json<Input>, ring: &State<Arc<Mutex<ConsistentHashing>>>) -> Json<GetNodeOutput> {
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
    let ring = ConsistentHashing::new(101);
    let ring = Arc::new(Mutex::new(ring));

    let cluster_name = String::from("value");
    let task_family = String::from("value");

    // checking if there are leafs running (in case dba is restarting)
    let ips = match get_ecs_task_private_ips(&ecs, &cluster_name, &task_family).await {
        Ok(ip_vec) => ip_vec,
        Err(e) => {
            println!("Error: {:?}", e);
            println!("Service is empty");
            Vec::new()
        }
    };

    {
        let mut ring = ring.lock().unwrap();
        for ip in ips {
            ring.add_node(&ip).unwrap();
        }
    }

    rocket::build()
        .manage(ring.clone())
        .manage(ecs)
        .mount("/", routes![
            hello,
            get_node,
            add_node,
            add_task,
            remove_task,
        ])
}