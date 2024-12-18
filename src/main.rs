use std::fs;
use clap::{ Parser, Subcommand };

mod api;
mod api_broker;

use api_broker::display_message;
use vestaboard_local::widgets::text::{ get_text, get_text_from_file };
use vestaboard_local::widgets::weather::get_weather;
use vestaboard_local::widgets::jokes::get_joke;

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
        #[clap(short, long, help = "Path to the file containing the message")]
        file: Option<String>,
        #[clap(short, long, help = "The message to display in \"\"")]
        message: Option<Vec<String>>,
    },
    Weather,
    Jokes,
    Clear,
    SATWord,
}

fn print_message(message: Vec<String>) {
    println!("Vestaboard Display:");
    println!("|----------------------|");
    message.iter().for_each(|line| {
        let padded_line = format!("{:<22}", line);
        println!("|{}|", padded_line);
    });
    println!("|----------------------|");
}

#[tokio::main]
async fn main() {
    let mut vb_codes: [[u8; 22]; 6] = [[0; 22]; 6];
    let cli = Cli::parse();
    let mut test_mode = false;
    if cli.test {
        test_mode = true;
    }

    let message: Option<Vec<String>> = match &cli.command {
        Commands::Text { file, message } => {
            println!("{:?}", file);
            if let Some(file) = file {
                Some(get_text_from_file(file))
            } else if let Some(message) = message {
                Some(get_text(&message.join(" ")))
            } else {
                None
            }
        }
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
            let sat_word = match vestaboard_local::widgets::sat_words::get_sat_word() {
                Ok(word) => word,
                Err(e) => {
                    eprintln!("Error retrieving SAT word: {:?}", e);
                    vec!["error retrieving sat word".to_string()]
                }
            };
            println!("{:?}", sat_word);
            Some(sat_word)
        }
        _ => {
            println!("Command not implemented");
            return;
        }
    };

    // let message_text = message.unwrap();
    if let Some(msg) = message {
        if test_mode {
            print_message(msg);
            return;
        }
        match display_message(msg) {
            None => println!("Error: message contains invalid characters."),
            Some(code) => {
                vb_codes = code;
                println!("{:?}", code);
            }
        }
    }
    api::send_message(vb_codes).await.unwrap();
}
