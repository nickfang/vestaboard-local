use std::fs;
use clap::{ Parser, Subcommand };

mod api;
mod message;
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
}

#[tokio::main]
async fn main() {
    let mut vb_codes: [[u8; 22]; 6] = [[0; 22]; 6];
    let cli = Cli::parse();
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
        _ => {
            println!("Command not implemented");
            return;
        }
    };

    if let Some(msg) = message {
        match message::convert_message(msg) {
            None => println!("Error: message contains invalid characters."),
            Some(code) => {
                vb_codes = code;
                println!("{:?}", code);
            }
        }
    }
    api::send_message(vb_codes).await.unwrap();
}
