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

pub fn convert_message(message: Vec<String>) -> Option<[[u8; 22]; 6]> {
    let mut formatted_message: [[u8; 22]; 6] = [[0; 22]; 6];
    let mut current_line = [0; 22];
    let mut line_num = 0;

    for line in message {
        if line_num == 6 {
            break;
        }
        let line_codes = to_codes(&line)?;
        if line_codes.len() > 22 {
            eprintln!("Too many characters on line {:?}", line_num);
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

    Some(formatted_message)
}
