use reqwest::Client;
use rocket::{post, serde::json::Json};
use serde_json::Value;
use std::{collections::HashMap, env, error::Error, mem::transmute, net::UdpSocket};

#[macro_use]
extern crate rocket;

#[post("/", data = "<write_req>")]
fn write_data(write_req: Json<Value>) -> String {

    let hostname = env::var("HOSTNAME").unwrap_or("default_value".to_string());

    println!("obj: {:?}", write_req);
    format!("Data written successfully to {}", hostname)
}

fn get_private_ip() -> Result<String, Box<dyn Error>> {
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.connect("8.8.8.8:80")?;
    let local_addr = socket.local_addr()?;
    Ok(local_addr.ip().to_string())
}

#[launch]
async fn rocket() -> _ {
    println!("Starting Rocket server...");

    println!("{:?}", get_private_ip());

    let client = Client::new();

    let ip = get_private_ip().unwrap();

    let mut payload = HashMap::new();
    payload.insert("value", &ip);
    
    let consistent_hashing_ip = env::var("CONSISTENT_HASHING_IP").unwrap();
    let uri = format!("http://{consistent_hashing_ip}:8000/add-node");

    let response = client.post(&uri)
        .json(&payload)
        .send().await.unwrap();

    let json_string = response.text().await.unwrap();

    let x: Vec<Value> = serde_json::from_str(&json_string).unwrap();

    for trans in x {
        if let Value::Object(map) = trans {
            println!("{:?}", map);
        }
    }

    rocket::build()
        .mount("/", routes![write_data])
}