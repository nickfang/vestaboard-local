mod api;
mod message;
mod widgets;

#[tokio::main]
async fn main() {
    // let note = vec!["hello", "", "world", "R O Y G B V ", "                 test", "!", "test"];
    let note = vec![
        "todos: ",
        "- audition for looma.",
        "- cook food for week.",
        "- laundry.",
        "- throw away trash.",
        "- rest."
    ];
    match message::convert_message(note) {
        None => println!("Error: message contains invalid characters."),
        Some(code) => {
            println!("{:?}", code);
            api::send_message(&code).await.unwrap();
        }
    }
    // widgets::jokes::send().await;
    // let weather_description = widgets::weather::get_weather().await.unwrap().to_lowercase();
    // match message::format_message(&weather_description) {
    //     None => println!("Error: message contains invalid characters."),
    //     Some(code) => {
    //         println!("{:?}", code);
    //         api::send_message(&code).await.unwrap();
    //     }
    // }
    // println!("{:?}", code);
}
