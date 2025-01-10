use clap::{ Parser, Subcommand };

mod api;
mod api_broker;
mod cli_display;
mod widgets;

use api_broker::display_message;
use cli_display::print_message;
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
        #[clap(help = "The message to display in \"\"")]
        message: Option<Vec<String>>,
    },
    File {
        #[clap(help = "The filename to read the message from")]
        name: String,
    },
    Weather,
    Jokes,
    Clear,
    SATWord,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let mut test_mode = false;
    if cli.test {
        test_mode = true;
    }

    let message: Option<Vec<String>> = match &cli.command {
        Commands::Text { message } => { Some(get_text(&message.clone().unwrap().join(" "))) }
        Commands::File { name } => { Some(get_text_from_file(name)) }
        Commands::Weather => {
            let weather_description = get_weather().await.unwrap();
            Some(weather_description)
        }
        Commands::Jokes => {
            let joke = get_joke();
            Some(joke)
        }
        Commands::Clear => {
            let clear = vec!["".to_string(); 6];
            Some(clear)
        }
        Commands::SATWord => {
            let sat_word = match get_sat_word() {
                Ok(word) => word,
                Err(e) => {
                    eprintln!("Error retrieving SAT word: {:?}", e);
                    vec!["error retrieving sat word".to_string()]
                }
            };
            println!("{:?}", sat_word);
            Some(sat_word)
        }
        // Catch for if an enum is added and a match arm is not created to handle it
        #[allow(unreachable_patterns)]
        _ => None, // Handle any future enum variants
    };
    if let Some(msg) = message {
        let display = display_message(msg.clone());
        match display {
            None => println!("Error: message contains invalid characters."),
            Some(code) => {
                if test_mode {
                    print_message(msg);
                    return;
                }
                api::send_message(code).await.unwrap();
            }
        }
    }
}

#[cfg(test)]
mod tests;
