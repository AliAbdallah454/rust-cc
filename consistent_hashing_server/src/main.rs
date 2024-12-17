use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket::{tokio, State};
use aws_config::{meta::region::RegionProviderChain, BehaviorVersion};
use consisten_hashing_server::ecs_functions::{get_ecs_task_private_ips, launch_task, stop_task};
use serde_json::Value;
use consisten_hashing_server::exclusives::RedirectInfo;
use consisten_hashing_server::utils::{check_alive, get_private_ip};
use consistent_hasher::{LDB, Identifier, Transaction};

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

#[post("/redirect", format = "json", data = "<input>")]
async fn redirect(input: Json<RedirectInfo>) {

    println!("In redirect");

    let redirect_info = input.into_inner();

    println!("{:?}", &redirect_info.exclusive);

    let destination = u128_to_string(redirect_info.destination);

    let client = reqwest::Client::new();
    let destination_url = format!("http://{}:7000/exclusive", &destination);
    client.post(destination_url).json(&redirect_info.exclusive).send().await.unwrap();

}

#[post("/remove-node/<ip>")]
async fn remove_node(ip: &str, ring: &State<Arc<Mutex<LDB>>>, ecs: &State<aws_sdk_ecs::Client>) -> Json<Vec<Transaction<u128>>> {

    let ring_ref = Arc::clone(ring);

    let node_to_delete = Leaf {
        ip: ip.to_string()
    };

    let transactions = {
        let mut ring = ring_ref.lock().unwrap();
        let transactions = ring.delete_node(node_to_delete).unwrap();
        drop(ring);
        transactions
    };

    if transactions.is_none() {
        println!("Transactions is None");
        return Json(Vec::new());
    }

    return Json(transactions.unwrap());

}

#[post("/add-node", format = "json", data = "<input>")]
async fn add_node(input: Json<Input>, ring: &State<Arc<Mutex<LDB>>>) -> Json<Value> {

    let input_value = input.value.clone();
    println!("Adding node with ip: {}", input_value);
    let ring_ref = Arc::clone(ring);
    let mut ring = ring_ref.lock().expect("Failed to lock the consistent hashing ring");
    
    let node_to_add = Leaf {
        ip: input_value,
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
        tokio::spawn(async move {

            println!("Current transactions: ");
            for transaction in &transactions {
                println!("{:?}", transaction);
            }

            let source = key.0;
            let destination = key.1;

            let source_ip = u128_to_string(source);
            let destination_ip = u128_to_string(destination);

            let client = reqwest::Client::new();

            let source_uri = format!("http://{}:7000/transactions", &source_ip);
            let get_response = client.post(&source_uri).json(&transactions).send().await.expect("this should not panic");

            if get_response.status() != 200 {
                println!("Unable to get exclusive");
                return;
            }

            let data = get_response.text().await.unwrap();
            let json_data: Value = serde_json::from_str(&data).unwrap();
            let (exclusive, token): (Value, String) = serde_json::from_value(json_data).unwrap();

            println!("{:?}", exclusive);
            
            let destination_uri = format!("http://{}:7000/exclusive", &destination_ip);
            let mut post_response = None;
            for _ in 0..4 {
                post_response = Some(client.post(&destination_uri).json(&exclusive).send().await);
                if post_response.as_ref().unwrap().is_ok() {
                    break;
                }
                println!("Connection refused for {}, retrying", &destination_ip);
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
    let node = u128_to_string(res.0);
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

fn u128_to_string(mut number: u128) -> String {
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
    // let ring = ConsistentHashing::new(6);
    // let ring = Arc::new(Mutex::new(ring));

    let ring = LDB::new(64, 5);
    let ring = Arc::new(Mutex::new(ring));

    // let cluster_name = String::from("value");
    // let task_family = String::from("value");

    // checking if there are leafs running (in case dba is restarting)
    // let ips = match get_ecs_task_private_ips(&ecs, &cluster_name, &task_family).await {
    //     Ok(ip_vec) => {
    //         println!("Adding: {:?}", &ip_vec);
    //         ip_vec
    //     },
    //     Err(e) => {
    //         println!("Error: {:?}", e);
    //         println!("Service is empty");
    //         Vec::new()
    //     }
    // };

    // {
    //     let mut ring = ring.lock().unwrap();
    //     let mut handles = vec![];
    //     for ip in ips {
    //         let handle = tokio::spawn(async move {
    //             if check_alive(&ip).await {
    //                 ring.add_node(&ip).unwrap();
    //                 println!("Added {}", &ip);
    //             }
    //             else {
    //                 println!("Failed to add {}", &ip);
    //             }
    //         });
    //         handles.push(handle);
    //     }
        
    //     for handle in handles {
    //         handle.await.unwrap();
    //     }
    // }

    rocket::build()
        .manage(ring.clone())
        .manage(ecs)
        .mount("/", routes![
            hello,
            get_node,
            add_node,
            add_task,
            remove_node,
            redirect
        ])
}