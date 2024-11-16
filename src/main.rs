mod api;
use std::{ thread, time::Duration };
use std::env;
use dotenv::dotenv;

fn main() {
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
            [0, 0, 0, 0, 0, 8, 5, 12, 12, 15, 0, 23, 15, 18, 12, 4, 37, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
        ]
        // Add more messages here
    ];

    let delay = Duration::from_secs(2); // Set your delay here

    for message in messages {
        api::send_message(api_key.clone(), &message);
        thread::sleep(delay);
    }
}
