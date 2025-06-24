use crate::widgets::widget_utils;
use crate::errors::VestaboardError;

pub fn get_joke() -> Result<Vec<String>, VestaboardError> {
    let joke = "what did the janitor say when he jumped out of the closet? \"supplies!\"";
    match widget_utils::format_message(joke) {
        None =>
            Err(
                VestaboardError::widget_error(
                    "jokes",
                    "Joke contains invalid characters for Vestaboard display"
                )
            ),
        Some(lines) => Ok(lines),
    }
}
