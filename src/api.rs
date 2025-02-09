use reqwest::ClientBuilder;
use serde_json::json;
use std::env;
use std::time::Duration;
use dotenv::dotenv;
use once_cell::sync::Lazy;

static API_KEY: Lazy<String> = Lazy::new(|| {
    dotenv().ok();
    env::var("LOCAL_API_KEY").expect("LOCAL_API_KEY not set")
});
static IP_ADDRESS: Lazy<String> = Lazy::new(|| {
    dotenv().ok();
    env::var("IP_ADDRESS").expect("IP_ADDRESS not set")
});

// Define the Api trait
pub trait Api {
    async fn send_message(&self, message: [[u8; 22]; 6]) -> Result<(), reqwest::Error>;
    async fn get_message(&self) -> Result<(), reqwest::Error>;
}

// Implement a concrete LocalApi
pub struct LocalApi;

impl LocalApi {
    pub fn new() -> Self {
        LocalApi
    }
}

impl Api for LocalApi {
    async fn send_message(&self, message: [[u8; 22]; 6]) -> Result<(), reqwest::Error> {
        let client = ClientBuilder::new().timeout(Duration::from_secs(10)).build()?;
        let url = format!("http://{}:7000/local-api/message", &*IP_ADDRESS);
        let body = json!(message);
        let res = client
            .post(&url)
            .header("X-Vestaboard-Local-Api-Key", &*API_KEY)
            .json(&body)
            .send().await;

        match res {
            Ok(response) => {
                println!("API: Response: {:?}", response);
                Ok(())
            }
            Err(e) => {
                eprintln!("API Error: {:?}", e);
                Err(e)
            }
        }
    }

    async fn get_message(&self) -> Result<(), reqwest::Error> {
        let client = ClientBuilder::new().timeout(Duration::from_secs(10)).build()?;
        let url = format!("http://{}:7000/local-api/message", &*IP_ADDRESS);

        let res = client.get(&url).header("X-Vestaboard-Local-Api-Key", &*API_KEY).send().await;

        match res {
            Ok(response) => {
                println!("API: Response: {:?}", response);
                Ok(())
            }
            Err(e) => {
                eprintln!("API: Error: {:?}", e);
                Err(e)
            }
        }
    }
}

// Helper functions now take a reference to anything implementing Api
#[allow(dead_code)]
pub async fn clear_board(api: &impl Api) -> Result<(), reqwest::Error> {
    let message = [[0; 22]; 6];
    api.send_message(message).await
}

#[allow(dead_code)]
pub async fn blank_board(api: &impl Api) -> Result<(), reqwest::Error> {
    let message = [[70; 22]; 6];
    api.send_message(message).await
}
