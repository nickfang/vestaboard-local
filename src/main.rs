use clap::{ Parser, Subcommand };

mod api;
mod api_broker;
mod cli_display;
mod scheduler;
mod widgets;

use api_broker::display_message;
use cli_display::print_message;
use widgets::text::{ get_text, get_text_from_file };
use widgets::weather::get_weather;
use widgets::jokes::get_joke;
use widgets::sat_words::get_sat_word;

#[derive(Subcommand, Debug)]
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
}
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

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let mut test_mode = false;
    if cli.test {
        test_mode = true;
    }

    let message: Vec<String> = match &cli.command {
        Commands::Text { message } => { get_text(&message.join(" ")) }
        Commands::File { name } => { get_text_from_file(name) }
        Commands::Weather => { get_weather().await }
        Commands::Jokes => { get_joke() }
        Commands::SATWord => { get_sat_word() }
        Commands::Clear => {
            api::clear_board().await.unwrap();
            return;
        }
    };
    match display_message(message.clone()) {
        None => {
            eprintln!("Error: message contains invalid characters.");
            // TODO: get formatted error message to send to vestaboard
        }
        Some(code) => {
            if test_mode {
                print_message(message);
                return;
            }
            api::send_message(code).await.unwrap();
        }
    }
}

#[cfg(test)]
mod tests;
