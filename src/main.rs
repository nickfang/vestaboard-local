mod api;
mod message;
mod jokes;
mod weather;

#[tokio::main]
async fn main() {
    // jokes::send().await;
    let weather_description = weather::get_weather().await.unwrap().to_lowercase();
    match message::format_message(&weather_description) {
        None => println!("Error: message contains invalid characters."),
        Some(code) => {
            println!("{:?}", code);
            api::send_message(&code).await.unwrap();
        }
    }
    // println!("{:?}", code);
}
