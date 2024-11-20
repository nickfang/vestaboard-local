mod api;
mod message;
use tokio::time::Duration;

// async fn send_message() {
//     let messages = vec![
//         vec![
//             [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
//             [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
//             [0, 0, 0, 0, 0, 8, 5, 12, 12, 15, 0, 23, 15, 18, 12, 4, 0, 0, 0, 0, 0, 0],
//             [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
//             [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
//             [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
//         ]
//     ];
//     for message in messages {
//         match api::send_message(api_key.clone(), &message).await {
//             Ok(_) => (),
//             Err(e) => eprintln!("Error sending message: {}", e),
//         }
//         tokio::time::sleep(Duration::from_secs(2)).await;
//     }
//     match api::clear_board(api_key.clone()).await {
//         Ok(_) => (),
//         Err(e) => eprintln!("Error clearing board: {}", e),
//     }
//     match api::blank_board(api_key.clone()).await {
//         Ok(_) => (),
//         Err(e) => eprintln!("Error blanking board: {}", e),
//     }
// }

#[tokio::main]
async fn main() {
    let joke = "what did the janitor say when he jumped out of the closet? \"supplies!\"";
    let colors =
        "R R R R R R R R R R R O O O O O O O O O O O Y Y Y Y Y Y Y Y Y Y Y G G G G G G G G G G G B B B B B B B B B B B V V V V V V V V V V V";
    match message::format_message(joke) {
        None => println!("Error: message contains invalid characters."),
        Some(code) => {
            println!("{:?}", code);
            api::send_message(&code).await.unwrap();
        }
    }
    // println!("{:?}", code);
}
