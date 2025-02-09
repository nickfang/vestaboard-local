use std::collections::HashMap;
use std::fs::File;
use std::io::{ self, BufRead };
use std::path::Path;
use serde::{ Deserialize, Serialize };
use rand::{ thread_rng, Rng };

use crate::widgets::widget_utils::{ format_error, split_into_lines, WidgetOutput };

#[derive(Serialize, Deserialize, Debug)]
struct UsedWord {
    word: String,
    timestamp: String,
}

pub fn get_sat_word() -> WidgetOutput {
    let path = "./src/widgets/sat_words/words.txt";
    let words_map = match create_word_map(path) {
        Ok(map) => map,
        Err(e) => {
            eprintln!("Error reading file: {:?}", e);
            return format_error("could not read file.");
        }
    };
    // let used_words_path = "./src/widgets/sat_words/used_words.json";
    // let mut used_words = load_used_words(used_words_path);
    // let used_words = match used_words {
    let mut rng = thread_rng();
    if let Some((key, value)) = words_map.iter().nth(rng.gen_range(0..words_map.len())) {
        let mut message = vec![format!("{} ({}):", key.to_string(), value[0].0.clone())];
        message.push("".to_string());
        let lines = split_into_lines(&value[0].1);
        for line in lines {
            message.push(line);
        }
        return message;
    }
    vec!["".to_string(), "".to_string(), "".to_string(), "".to_string(), "".to_string()]
}

fn create_word_map<P>(filename: P) -> io::Result<HashMap<String, Vec<(String, String, String)>>>
    where P: AsRef<Path>
{
    let file = File::open(filename)?;
    let reader = io::BufReader::new(file);
    let mut map = HashMap::new();

    for (line_number, line) in reader.lines().enumerate() {
        let line = line?;
        let lowercased_line = line.to_lowercase();
        let mut parts = lowercased_line.splitn(2, ' ');
        if let (Some(key), Some(rest)) = (parts.next(), parts.next()) {
            let mut definitions = Vec::new();
            let mut rest = rest.trim();
            while let Some((word_type, rest_of_line)) = rest.split_once(')') {
                if let Some((description, example)) = rest_of_line.split_once('(') {
                    definitions.push((
                        word_type[1..].to_string(), // remove paired ( from split_once
                        description.trim().to_string(),
                        example.trim().trim_end_matches(')').to_string(),
                    ));
                    rest = "";
                    // rest = rest_of_line.split_once(')').map_or("", |(_, r)| r.trim());
                } else {
                    println!(
                        "Line {} does not follow the expected pattern: {}",
                        line_number + 1,
                        line
                    );
                    break;
                }
            }
            if !definitions.is_empty() {
                map.insert(key.to_string(), definitions);
            }
        } else {
            println!("Line {} does not follow the expected pattern: {}", line_number + 1, line);
        }
    }

    Ok(map)
}

// fn load_used_words<P>(filename: P) -> io::Result<Vec<UsedWord>>
//     where P: AsRef<Path> + AsRef<std::ffi::OsStr>
// {
//     if !Path::new(&filename).exists() {
//         let file = File::create(&filename)?;
//         serde_json::to_writer(file, &Vec::<UsedWord>::new())?;
//         return Ok(Vec::new());
//     }
//     let file = File::open(filename)?;
//     let reader = io::BufReader::new(file);
//     let used_words: Vec<UsedWord> = serde_json::from_reader(reader)?;
//     Ok(used_words)
// }

// fn save_used_words<P>(filename: P, used_words: &Vec<UsedWord>) -> io::Result<()>
//     where P: AsRef<Path>
// {
//     let file = OpenOptions::new().write(true).create(true).truncate(true).open(filename)?;
//     serde_json::to_writer(file, used_words)?;
//     Ok(())
// }
