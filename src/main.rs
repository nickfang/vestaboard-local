mod api;
mod message;
use std::env;
use dotenv::dotenv;
use tokio::time::Duration;

#[tokio::main]
async fn main() {
    dotenv().ok();
    if env::var("LOCAL_API_KEY").is_err() {
        eprintln!("Error: LOCAL_API_KEY environment variable is not set.");
        std::process::exit(1);
    }
    let api_key = env::var("LOCAL_API_KEY").expect("LOCAL_API_KEY not set");

    let messages = vec![
        vec![
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 8, 5, 12, 12, 15, 0, 23, 15, 18, 12, 4, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
        ]
    ];
    for message in messages {
        match api::send_message(api_key.clone(), &message).await {
            Ok(_) => (),
            Err(e) => eprintln!("Error sending message: {}", e),
        }
        tokio::time::sleep(Duration::from_secs(2)).await;
    }
    match api::clear_board(api_key.clone()).await {
        Ok(_) => (),
        Err(e) => eprintln!("Error clearing board: {}", e),
    }
    match api::blank_board(api_key.clone()).await {
        Ok(_) => (),
        Err(e) => eprintln!("Error blanking board: {}", e),
    }
}
