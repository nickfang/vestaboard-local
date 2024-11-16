use reqwest::blocking::Client;
use serde_json::json;
use std::env;

pub fn send_message(api_key: String, message: &Vec<[i32; 22]>) {
    let client = Client::new();
    let ip_address = env::var("IP_ADDRESS").expect("IP_ADDRESS not set");
    let url = format!("http://{}:7000/local-api/message", ip_address);
    let body = json!(message);

    let res = client.post(&url).header("X-Vestaboard-Local-Api-Key", api_key).json(&body).send();

    match res {
        Ok(response) => println!("Response: {:?}", response),
        Err(e) => println!("Error: {:?}", e),
    }
}

pub fn get_message(api_key: String) {
    let client = Client::new();
    let ip_address = env::var("IP_ADDRESS").expect("IP_ADDRESS not set");
    let url = format!("http://{}:7000/local-api/message", ip_address);

    let res = client.get(&url).header("X-Vestaboard-Local-Api-Key", api_key).send();

    match res {
        Ok(response) =>
            match response.text() {
                Ok(text) => println!("Response: {:?}", text),
                Err(e) => println!("Error reading response text: {:?}", e),
            }
        Err(e) => println!("Error: {:?}", e),
    }
}
