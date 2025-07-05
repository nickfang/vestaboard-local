use crate::errors::VestaboardError;
use crate::widgets::widget_utils;

pub fn get_joke() -> Result<Vec<String>, VestaboardError> {
    log::debug!("Jokes widget starting");
    let joke = "what did the janitor say when he jumped out of the closet? \"supplies!\"";
    log::info!("Selected joke: {}", joke);
    // Widget just formats the message - validation happens at the main level
    let formatted = widget_utils::format_message(joke);
    log::debug!("Jokes widget completed successfully, {} lines generated", formatted.len());
    Ok(formatted)
}
