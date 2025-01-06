use clap::{ Parser, Subcommand };

mod api;
mod api_broker;
mod widgets;

use api_broker::display_message;
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

pub fn print_message(message: Vec<String>) {
    println!("Vestaboard Display:");
    println!("|----------------------|");
    message
        .iter()
        .take(6)
        .for_each(|line| {
            let padded_line = format!("{:<22}", line);
            const SOLID_SQUARE: char = '\u{2588}';
            let modified_line = padded_line
                .chars()
                .map(|c| {
                    match c {
                        'R' => format!("\x1b[{}m{}\x1b[0m", "31", SOLID_SQUARE),
                        'O' => format!("\x1b[{}m{}\x1b[0m", "38:5:208", SOLID_SQUARE),
                        'Y' => format!("\x1b[{}m{}\x1b[0m", "33", SOLID_SQUARE),
                        'G' => format!("\x1b[{}m{}\x1b[0m", "32", SOLID_SQUARE),
                        'B' => format!("\x1b[{}m{}\x1b[0m", "34", SOLID_SQUARE),
                        'V' => format!("\x1b[{}m{}\x1b[0m", "35", SOLID_SQUARE),
                        'W' => format!("\x1b[{}m{}\x1b[0m", "37", SOLID_SQUARE),
                        'K' => format!("\x1b[{}m{}\x1b[0m", "30", SOLID_SQUARE),
                        _ => c.to_string(),
                    }
                })
                .collect::<String>();
            println!("|{}|", modified_line);
        });
    println!("|----------------------|");
}

#[tokio::main]
async fn main() {
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
        match display_message(msg.clone()) {
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
