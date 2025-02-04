use clap::{ Parser, Subcommand };
use std::thread::sleep;
use std::time::Duration;

mod api;
mod api_broker;
mod cli_display;
mod widgets;

use crate::api_broker::{ ApiBroker, LocalApiBroker };
use widgets::text::{ get_text, get_text_from_file };
use widgets::weather::get_weather;
use widgets::jokes::get_joke;
use widgets::sat_words::get_sat_word;

#[derive(Parser)]
#[clap(
    name = "Vestaboard CLI",
    version = "1.0",
    author = "Nicholas Fang",
    about = "CLI for updating a local Vestaboard"
)]
struct Cli {
    #[clap(short, long, help = "Show message without sending to board.")]
    test: bool,

    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Text {
        #[clap(help = "The message to display", required = true)]
        message: Vec<String>,
    },
    File {
        #[clap(help = "The filename to read the message from", required = true, index = 1)]
        name: String,
    },
    Weather,
    Jokes,
    Clear,
    SATWord,
    Multiple,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let mut test_mode = false;
    if cli.test {
        test_mode = true;
    }

    let api_broker = LocalApiBroker::new();

    match &cli.command {
        Commands::Text { message } => {
            let message = get_text(&message.join(" "));
            api_broker.display_message(message, test_mode).await;
        }
        Commands::File { name } => {
            let message = get_text_from_file(name);
            api_broker.display_message(message, test_mode).await;
        }
        Commands::Weather => {
            let message = get_weather().await;
            api_broker.display_message(message, test_mode).await;
        }
        Commands::Jokes => {
            let message = get_joke();
            api_broker.display_message(message, test_mode).await;
        }
        Commands::SATWord => {
            let message = get_sat_word();
            api_broker.display_message(message, test_mode).await;
        }
        Commands::Clear => {
            let empty_message = vec!["".to_string()];
            api_broker.display_message(empty_message, test_mode).await;
            return;
        }
        Commands::Multiple => {
            let message1 = get_text_from_file("text1.txt");
            api_broker.display_message(message1, test_mode).await;
            sleep(Duration::from_secs(4)); // Add a 4-second delay

            let message2 = get_text_from_file("text2.txt");
            api_broker.display_message(message2, test_mode).await;
            sleep(Duration::from_secs(4)); // Add a 4-second delay

            let message3 = get_text_from_file("text3.txt");
            api_broker.display_message(message3, test_mode).await;
            sleep(Duration::from_secs(4)); // Add a 4-second delay

            let message4 = get_text_from_file("text4.txt");
            api_broker.display_message(message4, test_mode).await;

            return;
        }
    };
}

#[cfg(test)]
mod tests;
