use reqwest::Client;
use serde_json::json;
use std::env;

pub async fn send_message(api_key: String, message: &Vec<[i32; 22]>) -> Result<(), reqwest::Error> {
    let client = Client::new();
    let ip_address = env::var("IP_ADDRESS").expect("IP_ADDRESS not set");
    let url = format!("http://{}:7000/local-api/message", ip_address);
    let body = json!(message);

    let res = client
        .post(&url)
        .header("X-Vestaboard-Local-Api-Key", api_key)
        .json(&body)
        .send().await;

    match res {
        Ok(response) => {
            println!("Response: {:?}", response);
            Ok(())
        }
        Err(e) => {
            println!("Error: {:?}", e);
            Err(e)
        }
    }
}

pub async fn clear_board(api_key: String) -> Result<(), reqwest::Error> {
    let message = vec![[0; 22]; 6];
    send_message(api_key, &message).await
}

pub async fn blank_board(api_key: String) -> Result<(), reqwest::Error> {
    let message = vec![[70; 22]; 6];
    send_message(api_key, &message).await
}

pub async fn get_message(api_key: String) -> Result<(), reqwest::Error> {
    let client = Client::new();
    let ip_address = env::var("IP_ADDRESS").expect("IP_ADDRESS not set");
    let url = format!("http://{}:7000/local-api/message", ip_address);

    let res = client.get(&url).header("X-Vestaboard-Local-Api-Key", api_key).send().await;

    match res {
        Ok(response) => {
            println!("Response: {:?}", response);
            Ok(())
        }
        Err(e) => {
            println!("Error: {:?}", e);
            Err(e)
        }
    }
}
