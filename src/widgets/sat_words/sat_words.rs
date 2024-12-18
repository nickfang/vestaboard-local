use std::collections::HashMap;
use std::fs::{ File, OpenOptions };
use std::io::{ self, BufRead, Write };
use std::path::Path;
use serde::{ Deserialize, Serialize };
use chrono::Utc;

#[derive(Serialize, Deserialize, Debug)]
struct UsedWord {
    word: String,
    timestamp: String,
}

pub fn get_sat_word() -> Result<Vec<String>, io::Error> {
    let path = "./words.txt";
    let words_map = create_words_map(path)?;
    let used_words_path = "./used_words.json";
    let mut used_words = load_used_words(used_words_path)?;

    // Example usage: print the HashMap
    for (key, value) in &words_map {
        if key == "facile" {
            println!("{}: {:?}", key, value);
            if !used_words.iter().any(|w| w.word == *key) {
                used_words.push(UsedWord {
                    word: key.clone(),
                    timestamp: Utc::now().to_rfc3339(),
                });
                save_used_words(used_words_path, &used_words)?;
            }
            return Ok(
                vec![
                    format!("{}", value[0].0),
                    format!("{}", value[0].1),
                    format!("{}", value[0].2)
                ]
            );
        }
    }
    Ok(vec!["".to_string(), "".to_string(), "".to_string(), "".to_string(), "".to_string()])
}

fn create_words_map<P>(filename: P) -> io::Result<HashMap<String, Vec<(String, String, String)>>>
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
                        word_type.trim().to_string() + ")",
                        description.trim().to_string(),
                        example.trim().trim_end_matches(')').to_string(),
                    ));
                    rest = rest_of_line.split_once(')').map_or("", |(_, r)| r.trim());
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

fn load_used_words<P>(filename: P) -> io::Result<Vec<UsedWord>>
    where P: AsRef<Path> + AsRef<std::ffi::OsStr>
{
    if !Path::new(&filename).exists() {
        let file = File::create(&filename)?;
        serde_json::to_writer(file, &Vec::<UsedWord>::new())?;
        return Ok(Vec::new());
    }
    let file = File::open(filename)?;
    let reader = io::BufReader::new(file);
    let used_words: Vec<UsedWord> = serde_json::from_reader(reader)?;
    Ok(used_words)
}

fn save_used_words<P>(filename: P, used_words: &Vec<UsedWord>) -> io::Result<()>
    where P: AsRef<Path>
{
    let file = OpenOptions::new().write(true).create(true).truncate(true).open(filename)?;
    serde_json::to_writer(file, used_words)?;
    Ok(())
}
