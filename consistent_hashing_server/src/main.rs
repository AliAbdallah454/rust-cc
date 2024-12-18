use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket::{tokio, State};
use aws_config::{meta::region::RegionProviderChain, BehaviorVersion};
use consisten_hashing_server::ecs_functions::{get_ecs_task_private_ips, launch_task};
use serde_json::Value;
use consisten_hashing_server::exclusives::RedirectInfo;
use consisten_hashing_server::utils::get_private_ip;
use consistent_hasher::{LDB, Identifier, Transaction};

use consisten_hashing_server::exclusives::Exclusive;

#[macro_use] extern crate rocket;

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

#[derive(Serialize, Deserialize)]
struct TransactionDto {
    exclusive: Exclusive,
    token: String
}

#[post("/redirect", format = "json", data = "<input>")]
async fn redirect(input: Json<RedirectInfo>) {

    let redirect_info = input.into_inner();

    let destination = u128_to_ip(redirect_info.destination);
    let exclusive = redirect_info.exclusive;

    println!("reveived redirect to {}", &destination);

    let client = reqwest::Client::new();
    let destination_url = format!("http://{}:7000/exclusive", &destination);
    client.post(destination_url).json(&exclusive).send().await.unwrap();

}

#[post("/remove-node/<ip>")]
async fn remove_node(ip: &str, ring: &State<Arc<Mutex<LDB>>>, _ecs: &State<aws_sdk_ecs::Client>) -> Json<Vec<Transaction<u128>>> {

    println!("Received termination request from {}", ip);
    let ring_ref = Arc::clone(ring);

    let node_to_delete = Leaf {
        ip: ip.to_string()
    };

    let transactions = {
        let mut ring = ring_ref.lock().unwrap();
        let delete_output = ring.delete_node(node_to_delete);
        drop(ring);
        let transactions = match delete_output {
            Ok(transactions) => transactions,
            Err(e) => {
                println!("Error: {:?}", e);
                None
            }
        };
        transactions
    };

    if transactions.is_none() {
        println!("Transactions is None");
        return Json(Vec::new());
    }

    return Json(transactions.unwrap());

}

#[post("/add-node", format = "json", data = "<input>")]
async fn add_node(input: Json<Input>, ring: &State<Arc<Mutex<LDB>>>, 
    failed_exclusive: &State<Arc<Mutex<Vec<(Exclusive, String, String)>>>>) -> Json<Value> {

    let ip = input.value.clone();
    println!("Adding node with ip: {}", ip);
    let ring_ref = Arc::clone(ring);
    let mut ring = ring_ref.lock().expect("Failed to lock the consistent hashing ring");
    
    let node_to_add = Leaf {
        ip: ip,
    };

    let transactions = match ring.add_node(node_to_add) {
        Ok(transactions) => transactions.unwrap(),
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
        let key = (transaction.source, transaction.destination);
        groups
            .entry(key)
            .or_insert_with(Vec::new)
            .push(transaction);
    }

    for (key, transactions) in groups {

        let failed_exclusive_ref = Arc::clone(&failed_exclusive);

        tokio::spawn(async move {

            println!("Current transactions: ");
            for transaction in &transactions {
                println!("{:?}", transaction);
            }

            let source = key.0;
            let destination = key.1;

            let source_ip = u128_to_ip(source);
            let destination_ip = u128_to_ip(destination);

            let client = reqwest::Client::new();

            let source_uri = format!("http://{}:7000/transactions", &source_ip);
            let exclusive_fetch_response = client.post(&source_uri).json(&transactions).send().await.expect("this should not panic");

            if exclusive_fetch_response.status() != 200 {
                println!("Unable to get exclusive");
                return;
            }

            let transaction_dto: TransactionDto = exclusive_fetch_response.json().await.expect("Failed to parse response as TransactionDto");
            
            let exclusive = transaction_dto.exclusive;
            let token = transaction_dto.token;

            println!("Exclusive: {:?}", exclusive);
            println!("Token: {}", token);

            let destination_uri = format!("http://{}:7000/exclusive", &destination_ip);
            let mut exclusive_send_response = None;

            for _ in 0..10 {
                exclusive_send_response = Some(client.post(&destination_uri).json(&exclusive).send().await);
                if exclusive_send_response.as_ref().unwrap().is_ok() {
                    break;
                }
                println!("Connection refused for {}, retrying", &destination_ip);
                tokio::time::sleep(tokio::time::Duration::from_millis(250)).await;
            }

            if exclusive_send_response.is_none() {
                failed_exclusive_ref.lock().unwrap().push((exclusive, destination_ip.clone(), token.clone()));
                return;
            }

            // let exclusive_send_response = exclusive_send_response.unwrap().unwrap();
            // if exclusive_send_response.status() != 200 {
            //     println!("Exclusive wasn't ingested");
            //     return
            // }
            // println!("{:?}", exclusive_send_response);

            // let post_data = exclusive_send_response.text().await.unwrap();
            // println!("data: {}", post_data);

            let delete_uri = format!("http://{}:7000/delete-batch/{}", &source_ip, &token);
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

#[post("/get-node", format = "json", data = "<input>")]
async fn get_node(input: Json<Input>, ring: &State<Arc<Mutex<LDB>>>) -> Json<GetNodeOutput> {
    let input_value = input.value.clone();
    println!("Input val is: {}", input_value);
    let mut ring = ring.lock().expect("Failed to lock the consistent hashing ring");
    let res = ring.key(&input_value).unwrap();
    let node = u128_to_ip(res.0);
    let hash = res.1.to_string(); // parse hash to string
    
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

fn u128_to_ip(mut number: u128) -> String {
    let mut output = String::new();

    while number != 0 {
        let part = number % 1_000;
        number = number / 1_000;
        output.push_str(&part.to_string());
        output.push('.');
    }
    output.pop();

    let mut vectored: Vec<_> = output.split('.').collect();
    let n = vectored.len();

    for i in 0..2 {
        let temp = vectored[i];
        vectored[i] = vectored[n - i - 1];
        vectored[n - i - 1] = temp;
    }

    return vectored.join(".");

}

struct Leaf {
    pub ip: String,
}

impl Identifier for Leaf {
    fn identify(&self) -> usize {
        let ip_parts: Vec<String> = self.ip
            .split('.')
            .map(|part| format!("{:03}", part.parse::<u32>().unwrap_or(0)))
            .collect();

        let x = ip_parts.join("");
        let v = x.parse::<usize>().unwrap();
        v
    }
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

    let ring = LDB::new(64, 5);
    let ring = Arc::new(Mutex::new(ring));

    let cluster_name = String::from("egret-db-cluster");
    let service_name = String::from("leaf-service");

    // checking if there are leafs running (in case dba is restarting)
    let ips = match get_ecs_task_private_ips(&ecs, &cluster_name, &service_name).await {
        Ok(ip_vec) => {
            println!("Adding: {:?}", &ip_vec);
            ip_vec
        },
        Err(e) => {
            println!("Error: {:?}", e);
            println!("Service is empty");
            Vec::new()
        }
    };

    {
        let mut ring = ring.lock().unwrap();
        for ip in ips {

            let node_to_add = Leaf {
                ip: ip.clone()
            };

            ring.add_node(node_to_add).unwrap();
            println!("Added {}", &ip);
        }
        
    }

    let failed_exclusives: Arc<Mutex<Vec<(Exclusive, String, String)>>> = Arc::new(Mutex::new(Vec::new()));

    rocket::build()
        .manage(ring.clone())
        .manage(ecs)
        .manage(failed_exclusives)
        .mount("/", routes![
            hello,
            get_node,
            add_node,
            add_task,
            remove_node,
            redirect
        ])
}