use reqwest::Client;
use serde_json::json;
use std::env;
use dotenv::dotenv;
use lazy_static::lazy_static;

lazy_static! {
    static ref API_KEY: String = {
        dotenv().ok();
        env::var("LOCAL_API_KEY").expect("LOCAL_API_KEY not set")
    };
    static ref IP_ADDRESS: String = {
        dotenv().ok();
        env::var("IP_ADDRESS").expect("IP_ADDRESS not set")
    };
}

pub async fn send_message(message: [[u8; 22]; 6]) -> Result<(), reqwest::Error> {
    let client = Client::new();
    let url = format!("http://{}:7000/local-api/message", &*IP_ADDRESS);
    let body = json!(message);
    // return Ok(());
    let res = client
        .post(&url)
        .header("X-Vestaboard-Local-Api-Key", API_KEY.clone())
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

pub async fn clear_board() -> Result<(), reqwest::Error> {
    let message = [[0; 22]; 6];
    match send_message(message).await {
        Ok(_) => Ok(()),
        Err(e) => {
            println!("Error: {:?}", e);
            Err(e)
        }
    }
}

pub async fn blank_board() -> Result<(), reqwest::Error> {
    let message = [[70; 22]; 6];
    match send_message(message).await {
        Ok(_) => Ok(()),
        Err(e) => {
            println!("Error: {:?}", e);
            Err(e)
        }
    }
}

pub async fn get_message() -> Result<(), reqwest::Error> {
    let client = Client::new();
    let url = format!("http://{}:7000/local-api/message", &*IP_ADDRESS);

    let res = client.get(&url).header("X-Vestaboard-Local-Api-Key", API_KEY.clone()).send().await;

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
