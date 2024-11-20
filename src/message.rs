use std::collections::HashMap;
use lazy_static::lazy_static;

// #[derive(Hash, Eq, PartialEq, Debug)]
// enum Character {
//     Blank,
//     A,
//     B,
//     C,
//     D,
//     E,
//     F,
//     G,
//     H,
//     I,
//     J,
//     K,
//     L,
//     M,
//     N,
//     O,
//     P,
//     Q,
//     R,
//     S,
//     T,
//     U,
//     V,
//     W,
//     X,
//     Y,
//     Z,
//     Num1,
//     Num2,
//     Num3,
//     Num4,
//     Num5,
//     Num6,
//     Num7,
//     Num8,
//     Num9,
//     Num0,
//     Exclamation,
//     At,
//     Hash,
//     Dollar,
//     LeftParen,
//     RightParen,
//     Hyphen,
//     Plus,
//     Ampersand,
//     Equals,
//     Semicolon,
//     Colon,
//     SingleQuote,
//     DoubleQuote,
//     Percent,
//     Comma,
//     Period,
//     Slash,
//     Question,
//     Degree,
//     Red,
//     Orange,
//     Yellow,
//     Green,
//     Blue,
//     Violet,
//     White,
//     Black,
//     Filled,
// }

// struct CharacterInfo {
//     value: u8,
//     label: &'static str,
//     name: &'static str,
//     note: &'static str,
// }

// impl Character {
//     fn info(&self) -> CharacterInfo {
//         match self {
//             Character::Blank =>
//                 CharacterInfo { value: 0, label: " ", name: "Blank", note: "Blank" },
//             Character::A =>
//                 CharacterInfo { value: 1, label: "A", name: "Letter A", note: "Uppercase A" },
//             Character::B =>
//                 CharacterInfo { value: 2, label: "B", name: "Letter B", note: "Uppercase B" },
//             Character::C =>
//                 CharacterInfo { value: 3, label: "C", name: "Letter C", note: "Uppercase C" },
//             Character::D =>
//                 CharacterInfo { value: 4, label: "D", name: "Letter D", note: "Uppercase D" },
//             Character::E =>
//                 CharacterInfo { value: 5, label: "E", name: "Letter E", note: "Uppercase E" },
//             Character::F =>
//                 CharacterInfo { value: 6, label: "F", name: "Letter F", note: "Uppercase F" },
//             Character::G =>
//                 CharacterInfo { value: 7, label: "G", name: "Letter G", note: "Uppercase G" },
//             Character::H =>
//                 CharacterInfo { value: 8, label: "H", name: "Letter H", note: "Uppercase H" },
//             Character::I =>
//                 CharacterInfo { value: 9, label: "I", name: "Letter I", note: "Uppercase I" },
//             Character::J =>
//                 CharacterInfo { value: 10, label: "J", name: "Letter J", note: "Uppercase J" },
//             Character::K =>
//                 CharacterInfo { value: 11, label: "K", name: "Letter K", note: "Uppercase K" },
//             Character::L =>
//                 CharacterInfo { value: 12, label: "L", name: "Letter L", note: "Uppercase L" },
//             Character::M =>
//                 CharacterInfo { value: 13, label: "M", name: "Letter M", note: "Uppercase M" },
//             Character::N =>
//                 CharacterInfo { value: 14, label: "N", name: "Letter N", note: "Uppercase N" },
//             Character::O =>
//                 CharacterInfo { value: 15, label: "O", name: "Letter O", note: "Uppercase O" },
//             Character::P =>
//                 CharacterInfo { value: 16, label: "P", name: "Letter P", note: "Uppercase P" },
//             Character::Q =>
//                 CharacterInfo { value: 17, label: "Q", name: "Letter Q", note: "Uppercase Q" },
//             Character::R =>
//                 CharacterInfo { value: 18, label: "R", name: "Letter R", note: "Uppercase R" },
//             Character::S =>
//                 CharacterInfo { value: 19, label: "S", name: "Letter S", note: "Uppercase S" },
//             Character::T =>
//                 CharacterInfo { value: 20, label: "T", name: "Letter T", note: "Uppercase T" },
//             Character::U =>
//                 CharacterInfo { value: 21, label: "U", name: "Letter U", note: "Uppercase U" },
//             Character::V =>
//                 CharacterInfo { value: 22, label: "V", name: "Letter V", note: "Uppercase V" },
//             Character::W =>
//                 CharacterInfo { value: 23, label: "W", name: "Letter W", note: "Uppercase W" },
//             Character::X =>
//                 CharacterInfo { value: 24, label: "X", name: "Letter X", note: "Uppercase X" },
//             Character::Y =>
//                 CharacterInfo { value: 25, label: "Y", name: "Letter Y", note: "Uppercase Y" },
//             Character::Z =>
//                 CharacterInfo { value: 26, label: "Z", name: "Letter Z", note: "Uppercase Z" },
//             Character::Num1 =>
//                 CharacterInfo { value: 27, label: "1", name: "Number 1", note: "Digit 1" },
//             Character::Num2 =>
//                 CharacterInfo { value: 28, label: "2", name: "Number 2", note: "Digit 2" },
//             Character::Num3 =>
//                 CharacterInfo { value: 29, label: "3", name: "Number 3", note: "Digit 3" },
//             Character::Num4 =>
//                 CharacterInfo { value: 30, label: "4", name: "Number 4", note: "Digit 4" },
//             Character::Num5 =>
//                 CharacterInfo { value: 31, label: "5", name: "Number 5", note: "Digit 5" },
//             Character::Num6 =>
//                 CharacterInfo { value: 32, label: "6", name: "Number 6", note: "Digit 6" },
//             Character::Num7 =>
//                 CharacterInfo { value: 33, label: "7", name: "Number 7", note: "Digit 7" },
//             Character::Num8 =>
//                 CharacterInfo { value: 34, label: "8", name: "Number 8", note: "Digit 8" },
//             Character::Num9 =>
//                 CharacterInfo { value: 35, label: "9", name: "Number 9", note: "Digit 9" },
//             Character::Num0 =>
//                 CharacterInfo { value: 36, label: "0", name: "Number 0", note: "Digit 0" },
//             Character::Exclamation =>
//                 CharacterInfo {
//                     value: 37,
//                     label: "!",
//                     name: "Exclamation Mark",
//                     note: "Punctuation",
//                 },
//             Character::At =>
//                 CharacterInfo { value: 38, label: "@", name: "At Symbol", note: "Punctuation" },
//             Character::Hash =>
//                 CharacterInfo { value: 39, label: "#", name: "Hash Symbol", note: "Punctuation" },
//             Character::Dollar =>
//                 CharacterInfo { value: 40, label: "$", name: "Dollar Symbol", note: "Punctuation" },
//             Character::LeftParen =>
//                 CharacterInfo {
//                     value: 41,
//                     label: "(",
//                     name: "Left Parenthesis",
//                     note: "Punctuation",
//                 },
//             Character::RightParen =>
//                 CharacterInfo {
//                     value: 42,
//                     label: ")",
//                     name: "Right Parenthesis",
//                     note: "Punctuation",
//                 },
//             Character::Hyphen =>
//                 CharacterInfo { value: 44, label: "-", name: "Hyphen", note: "Punctuation" },
//             Character::Plus =>
//                 CharacterInfo { value: 46, label: "+", name: "Plus Sign", note: "Punctuation" },
//             Character::Ampersand =>
//                 CharacterInfo { value: 47, label: "&", name: "Ampersand", note: "Punctuation" },
//             Character::Equals =>
//                 CharacterInfo { value: 48, label: "=", name: "Equals Sign", note: "Punctuation" },
//             Character::Semicolon =>
//                 CharacterInfo { value: 49, label: ";", name: "Semicolon", note: "Punctuation" },
//             Character::Colon =>
//                 CharacterInfo { value: 50, label: ":", name: "Colon", note: "Punctuation" },
//             Character::SingleQuote =>
//                 CharacterInfo { value: 52, label: "'", name: "Single Quote", note: "Punctuation" },
//             Character::DoubleQuote =>
//                 CharacterInfo { value: 53, label: "\"", name: "Double Quote", note: "Punctuation" },
//             Character::Percent =>
//                 CharacterInfo {
//                     value: 54,
//                     label: "%",
//                     name: "Percent Symbol",
//                     note: "Punctuation",
//                 },
//             Character::Comma =>
//                 CharacterInfo { value: 55, label: ",", name: "Comma", note: "Punctuation" },
//             Character::Period =>
//                 CharacterInfo { value: 56, label: ".", name: "Period", note: "Punctuation" },
//             Character::Slash =>
//                 CharacterInfo { value: 59, label: "/", name: "Slash", note: "Punctuation" },
//             Character::Question =>
//                 CharacterInfo { value: 60, label: "?", name: "Question Mark", note: "Punctuation" },
//             Character::Degree =>
//                 CharacterInfo { value: 62, label: "°", name: "Degree", note: "Punctuation" },
//             Character::Red =>
//                 CharacterInfo { value: 63, label: "Red", name: "Red Block", note: "Color" },
//             Character::Orange =>
//                 CharacterInfo { value: 64, label: "Orange", name: "Orange Block", note: "Color" },
//             Character::Yellow =>
//                 CharacterInfo { value: 65, label: "Yellow", name: "Yellow Block", note: "Color" },
//             Character::Green =>
//                 CharacterInfo { value: 66, label: "Green", name: "Green Block", note: "Color" },
//             Character::Blue =>
//                 CharacterInfo { value: 67, label: "Blue", name: "Blue Block", note: "Color" },
//             Character::Violet =>
//                 CharacterInfo { value: 68, label: "Violet", name: "Violet Block", note: "Color" },
//             Character::White =>
//                 CharacterInfo { value: 69, label: "White", name: "White Block", note: "Color" },
//             Character::Black =>
//                 CharacterInfo { value: 70, label: "Black", name: "Black Block", note: "Color" },
//             Character::Filled =>
//                 CharacterInfo { value: 71, label: "Filled", name: "Filled Block", note: "Color" },
//         }
//     }
// }
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
