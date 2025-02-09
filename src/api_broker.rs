use std::collections::HashMap;
use once_cell::sync::Lazy;

use crate::api::{ Api, LocalApi };
use crate::cli_display::print_message;

static CHARACTER_CODES: Lazy<HashMap<char, u8>> = Lazy::new(|| {
    let characters = [
        (' ', 0),
        ('a', 1),
        ('b', 2),
        ('c', 3),
        ('d', 4),
        ('e', 5),
        ('f', 6),
        ('g', 7),
        ('h', 8),
        ('i', 9),
        ('j', 10),
        ('k', 11),
        ('l', 12),
        ('m', 13),
        ('n', 14),
        ('o', 15),
        ('p', 16),
        ('q', 17),
        ('r', 18),
        ('s', 19),
        ('t', 20),
        ('u', 21),
        ('v', 22),
        ('w', 23),
        ('x', 24),
        ('y', 25),
        ('z', 26),
        ('1', 27),
        ('2', 28),
        ('3', 29),
        ('4', 30),
        ('5', 31),
        ('6', 32),
        ('7', 33),
        ('8', 34),
        ('9', 35),
        ('0', 36),
        ('!', 37),
        ('@', 38),
        ('#', 39),
        ('$', 40),
        ('(', 41),
        (')', 42),
        ('-', 44),
        ('+', 46),
        ('&', 47),
        ('=', 48),
        (';', 49),
        (':', 50),
        ('\'', 52),
        ('"', 53),
        ('%', 54),
        (',', 55),
        ('.', 56),
        ('/', 59),
        ('?', 60),
        ('D', 62),
        ('R', 63),
        ('O', 64),
        ('Y', 65),
        ('G', 66),
        ('B', 67),
        ('V', 68),
        ('W', 69),
        ('K', 70),
        ('F', 71),
    ];
    characters.iter().cloned().collect()
});

pub fn get_valid_characters() -> HashMap<char, u8> {
    CHARACTER_CODES.clone()
}

// Define the ApiBroker trait
pub trait ApiBroker {
    async fn display_message(&self, message: Vec<String>, test_mode: bool);
}

pub struct LocalApiBroker<T: Api> {
    api: T,
}

impl<T: Api> LocalApiBroker<T> {
    pub fn new_with_api(api: T) -> Self {
        LocalApiBroker { api }
    }

    pub fn to_codes(&self, message: &str) -> Option<Vec<u8>> {
        let mut codes = Vec::new();
        let mut invalid_chars = Vec::new();

        for c in message.chars() {
            match CHARACTER_CODES.get(&c) {
                Some(&code) => codes.push(code),
                None => invalid_chars.push(c),
            }
        }

        if !invalid_chars.is_empty() {
            eprintln!("API_BROKER: Invalid characters found: {:?}", invalid_chars);
            // eprintln!("These characters have been removed from the message.");
            return None;
        }

        Some(codes)
    }
}

impl LocalApiBroker<LocalApi> {
    pub fn new() -> Self {
        LocalApiBroker { api: LocalApi::new() }
    }
}
impl<T: Api> ApiBroker for LocalApiBroker<T> {
    async fn display_message(&self, message: Vec<String>, test_mode: bool) {
        if test_mode {
            print_message(message);
            return;
        }

        let mut formatted_message: [[u8; 22]; 6] = [[0; 22]; 6];
        let mut current_line = [0; 22];
        let mut line_num = 0;

        for line in message {
            if line_num == 6 {
                break;
            }
            let line_codes = match self.to_codes(&line) {
                Some(codes) => codes,
                None => {
                    return;
                }
            };
            if line_codes.len() > 22 {
                eprintln!("API_BROKER: Too many characters on line {:?}", line_num);
            }
            // make sure and pad with 0's or characters on the previous line will be duplicated
            for i in 0..22 {
                if i < line_codes.len() {
                    current_line[i] = line_codes[i];
                } else {
                    current_line[i] = 0;
                }
            }
            formatted_message[line_num] = current_line;
            line_num += 1;
        }

        let api = LocalApi::new();
        match api.send_message(formatted_message).await {
            Ok(_) => {
                println!("API_BROKER: Message sent to Vestaboard.");
            }
            Err(e) => {
                eprintln!("API_BROKER: Error sending message: {:?}", e);
            }
        }
    }
}
