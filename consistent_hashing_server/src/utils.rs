use std::net::UdpSocket;

pub fn get_private_ip() -> Result<String, Box<dyn std::error::Error>> {
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.connect("8.8.8.8:80")?;
    let local_addr = socket.local_addr()?;
    Ok(local_addr.ip().to_string())
}

pub async fn check_alive(ip: &str) -> bool {

    let client = reqwest::Client::new();
    let target_leaf = format!("http://{}:7000/check-alive", ip);

    let response = client.get(target_leaf).send().await;
    response.is_ok()

}