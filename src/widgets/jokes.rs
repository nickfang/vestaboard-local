use vestaboard_local::api;
use vestaboard_local::message;

// use message::{ to_codes, format_message };
// use api::send_message;

pub async fn send() {
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
}
