use crate::errors::VestaboardError;
use crate::widgets::widget_utils;

pub fn get_joke() -> Result<Vec<String>, VestaboardError> {
    let joke = "what did the janitor say when he jumped out of the closet? \"supplies!\"";
    // Widget just formats the message - validation happens at the main level
    Ok(widget_utils::format_message(joke))
}
