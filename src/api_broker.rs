use std::collections::HashMap;
use once_cell::sync::Lazy;

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

pub fn display_message(message: Vec<String>) -> Option<[[u8; 22]; 6]> {
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
