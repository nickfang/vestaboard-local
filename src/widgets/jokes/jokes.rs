use crate::widgets::widget_utils;

pub fn get_joke() -> Vec<String> {
    let joke = "what did the janitor say when he jumped out of the closet? \"supplies!\"";
    match widget_utils::format_message(joke) {
        None => {
            eprintln!("Error: message contains invalid characters.");
            Vec::new()
        }
        Some(lines) => lines,
    }
}
