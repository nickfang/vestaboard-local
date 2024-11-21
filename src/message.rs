use std::collections::HashMap;
use lazy_static::lazy_static;

lazy_static! {
  static ref CHARACTER_CODES: HashMap<char, u8> = {
    let mut m = HashMap::new();
    m.insert(' ', 0);
    m.insert('a', 1);
    m.insert('b', 2);
    m.insert('c', 3);
    m.insert('d', 4);
    m.insert('e', 5);
    m.insert('f', 6);
    m.insert('g', 7);
    m.insert('h', 8);
    m.insert('i', 9);
    m.insert('j', 10);
    m.insert('k', 11);
    m.insert('l', 12);
    m.insert('m', 13);
    m.insert('n', 14);
    m.insert('o', 15);
    m.insert('p', 16);
    m.insert('q', 17);
    m.insert('r', 18);
    m.insert('s', 19);
    m.insert('t', 20);
    m.insert('u', 21);
    m.insert('v', 22);
    m.insert('w', 23);
    m.insert('x', 24);
    m.insert('y', 25);
    m.insert('z', 26);
    m.insert('1', 27);
    m.insert('2', 28);
    m.insert('3', 29);
    m.insert('4', 30);
    m.insert('5', 31);
    m.insert('6', 32);
    m.insert('7', 33);
    m.insert('8', 34);
    m.insert('9', 35);
    m.insert('0', 36);
    m.insert('!', 37);
    m.insert('@', 38);
    m.insert('#', 39);
    m.insert('$', 40);
    m.insert('(', 41);
    m.insert(')', 42);
    m.insert('-', 44);
    m.insert('+', 46);
    m.insert('&', 47);
    m.insert('=', 48);
    m.insert(';', 49);
    m.insert(':', 50);
    m.insert('"', 52);
    m.insert('"', 53);
    m.insert('%', 54);
    m.insert(',', 55);
    m.insert('.', 56);
    m.insert('/', 59);
    m.insert('?', 60);
    m.insert('°', 62);
    m.insert('R', 63);
    m.insert('O', 64);
    m.insert('Y', 65);
    m.insert('G', 66);
    m.insert('B', 67);
    m.insert('V', 68);
    m.insert('W', 69);
    m.insert('K', 70);
    m.insert('F', 71);
    m
  };
}

pub fn to_codes(message: &str) -> Option<Vec<u8>> {
    let mut codes = Vec::new();
    let mut invalid_chars = Vec::new();

    for c in message.chars() {
        match CHARACTER_CODES.get(&c) {
            Some(&code) => codes.push(code),
            None => invalid_chars.push(c),
        }
    }

    if !invalid_chars.is_empty() {
        eprintln!("Invalid characters found: {:?}", invalid_chars);
        // eprintln!("These characters have been removed from the message.");
        return None;
    }

    Some(codes)
}

fn center_line(line: &mut [i32; 22]) {
    let mut start = 0;
    let mut end = 21;
    while start < end && line[start] == 0 {
        start += 1;
    }
    while end > start && line[end] == 0 {
        end -= 1;
    }
    let len = end - start + 1;
    let padding = (22 - len) / 2;
    if padding > 0 {
        for i in (start..=end).rev() {
            if i + padding < 22 {
                line[i + padding] = line[i];
            }
        }
        for i in start..start + padding {
            if i < 22 {
                line[i] = 0;
            }
        }
        for i in end + padding + 1..22 {
            if i < 22 {
                line[i] = 0;
            }
        }
    }
}

fn center_message_vertically(message: &mut Vec<[i32; 22]>) {
    let vertical_padding = (6 - message.len()) / 2;
    println!("Vertical padding: {}", vertical_padding.clone());
    if vertical_padding > 0 {
        for _ in 0..vertical_padding {
            message.insert(0, [0; 22]);
        }
        while message.len() < 6 {
            message.push([0; 22]);
        }
    }
}

pub fn format_message(message: &str) -> Option<Vec<[i32; 22]>> {
    let mut formatted_message = Vec::new();
    let words: Vec<&str> = message.split_whitespace().collect();
    let mut current_line = [0; 22];
    let mut col = 0;

    for word in words {
        let word_codes = to_codes(word)?;
        if col + word_codes.len() > 22 {
            center_line(&mut current_line);
            formatted_message.push(current_line);
            current_line = [0; 22];
            col = 0;
        }
        if col + word_codes.len() <= 22 {
            for &code in &word_codes {
                current_line[col] = code as i32;
                col += 1;
            }
            if col < 22 {
                current_line[col] = 0; // Add space between words
                col += 1;
            }
        } else {
            // If a single word is longer than 22 characters, split it
            for &code in &word_codes {
                if col == 22 {
                    center_line(&mut current_line);
                    formatted_message.push(current_line);
                    current_line = [0; 22];
                    col = 0;
                }
                current_line[col] = code as i32;
                col += 1;
            }
        }
    }

    if col > 0 {
        center_line(&mut current_line);
        formatted_message.push(current_line);
    }

    center_message_vertically(&mut formatted_message);

    Some(formatted_message)
}
