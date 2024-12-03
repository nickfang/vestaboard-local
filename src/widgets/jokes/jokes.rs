use crate::widgets::widget_utils;

pub fn get_joke() -> Vec<String> {
    let joke = "what did the janitor say when he jumped out of the closet? \"supplies!\"";
    let colors =
        "R R R R R R R R R R R O O O O O O O O O O O Y Y Y Y Y Y Y Y Y Y Y G G G G G G G G G G G B B B B B B B B B B B V V V V V V V V V V V";
    match widget_utils::format_message(joke) {
        None => {
            println!("Error: message contains invalid characters.");
            Vec::new()
        }
        Some(lines) => lines,
    }
}
