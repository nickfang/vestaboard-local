use std::collections::HashMap;

#[derive(Hash, Eq, PartialEq, Debug)]
enum Character {
    Space,
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    Num1,
    Num2,
    Num3,
    Num4,
    Num5,
    Num6,
    Num7,
    Num8,
    Num9,
    Num0,
    Exclamation,
    At,
    Hash,
    Dollar,
    Percent,
    Caret,
    Ampersand,
    Asterisk,
    LeftParen,
    RightParen,
    Hyphen,
    Underscore,
    Equals,
    Plus,
    LeftBracket,
    RightBracket,
    LeftBrace,
    RightBrace,
    Backslash,
    Pipe,
    Semicolon,
    Colon,
    SingleQuote,
    DoubleQuote,
    Comma,
    Period,
    LessThan,
    GreaterThan,
    Slash,
    Question,
}

struct CharacterInfo {
    value: u8,
    label: &'static str,
    name: &'static str,
    note: &'static str,
}

impl Character {
    fn info(&self) -> CharacterInfo {
        match self {
            Character::A =>
                CharacterInfo { value: 1, label: "A", name: "Letter A", note: "Uppercase A" },
            Character::B =>
                CharacterInfo { value: 2, label: "B", name: "Letter B", note: "Uppercase B" },
            Character::C =>
                CharacterInfo { value: 3, label: "C", name: "Letter C", note: "Uppercase C" },
            Character::D =>
                CharacterInfo { value: 4, label: "D", name: "Letter D", note: "Uppercase D" },
            Character::E =>
                CharacterInfo { value: 5, label: "E", name: "Letter E", note: "Uppercase E" },
            Character::F =>
                CharacterInfo { value: 6, label: "F", name: "Letter F", note: "Uppercase F" },
            Character::G =>
                CharacterInfo { value: 7, label: "G", name: "Letter G", note: "Uppercase G" },
            Character::H =>
                CharacterInfo { value: 8, label: "H", name: "Letter H", note: "Uppercase H" },
            Character::I =>
                CharacterInfo { value: 9, label: "I", name: "Letter I", note: "Uppercase I" },
            Character::J =>
                CharacterInfo { value: 10, label: "J", name: "Letter J", note: "Uppercase J" },
            Character::K =>
                CharacterInfo { value: 11, label: "K", name: "Letter K", note: "Uppercase K" },
            Character::L =>
                CharacterInfo { value: 12, label: "L", name: "Letter L", note: "Uppercase L" },
            Character::M =>
                CharacterInfo { value: 13, label: "M", name: "Letter M", note: "Uppercase M" },
            Character::N =>
                CharacterInfo { value: 14, label: "N", name: "Letter N", note: "Uppercase N" },
            Character::O =>
                CharacterInfo { value: 15, label: "O", name: "Letter O", note: "Uppercase O" },
            Character::P =>
                CharacterInfo { value: 16, label: "P", name: "Letter P", note: "Uppercase P" },
            Character::Q =>
                CharacterInfo { value: 17, label: "Q", name: "Letter Q", note: "Uppercase Q" },
            Character::R =>
                CharacterInfo { value: 18, label: "R", name: "Letter R", note: "Uppercase R" },
            Character::S =>
                CharacterInfo { value: 19, label: "S", name: "Letter S", note: "Uppercase S" },
            Character::T =>
                CharacterInfo { value: 20, label: "T", name: "Letter T", note: "Uppercase T" },
            Character::U =>
                CharacterInfo { value: 21, label: "U", name: "Letter U", note: "Uppercase U" },
            Character::V =>
                CharacterInfo { value: 22, label: "V", name: "Letter V", note: "Uppercase V" },
            Character::W =>
                CharacterInfo { value: 23, label: "W", name: "Letter W", note: "Uppercase W" },
            Character::X =>
                CharacterInfo { value: 24, label: "X", name: "Letter X", note: "Uppercase X" },
            Character::Y =>
                CharacterInfo { value: 25, label: "Y", name: "Letter Y", note: "Uppercase Y" },
            Character::Z =>
                CharacterInfo { value: 26, label: "Z", name: "Letter Z", note: "Uppercase Z" },
            Character::Num1 =>
                CharacterInfo { value: 27, label: "1", name: "Number 1", note: "Digit 1" },
            Character::Num2 =>
                CharacterInfo { value: 28, label: "2", name: "Number 2", note: "Digit 2" },
            Character::Num3 =>
                CharacterInfo { value: 29, label: "3", name: "Number 3", note: "Digit 3" },
            Character::Num4 =>
                CharacterInfo { value: 30, label: "4", name: "Number 4", note: "Digit 4" },
            Character::Num5 =>
                CharacterInfo { value: 31, label: "5", name: "Number 5", note: "Digit 5" },
            Character::Num6 =>
                CharacterInfo { value: 32, label: "6", name: "Number 6", note: "Digit 6" },
            Character::Num7 =>
                CharacterInfo { value: 33, label: "7", name: "Number 7", note: "Digit 7" },
            Character::Num8 =>
                CharacterInfo { value: 34, label: "8", name: "Number 8", note: "Digit 8" },
            Character::Num9 =>
                CharacterInfo { value: 35, label: "9", name: "Number 9", note: "Digit 9" },
            Character::Num0 =>
                CharacterInfo { value: 36, label: "0", name: "Number 0", note: "Digit 0" },
            Character::Exclamation =>
                CharacterInfo {
                    value: 37,
                    label: "!",
                    name: "Exclamation Mark",
                    note: "Punctuation",
                },
            Character::At =>
                CharacterInfo { value: 38, label: "@", name: "At Symbol", note: "Punctuation" },
            Character::Hash =>
                CharacterInfo { value: 39, label: "#", name: "Hash Symbol", note: "Punctuation" },
            Character::Dollar =>
                CharacterInfo { value: 40, label: "$", name: "Dollar Symbol", note: "Punctuation" },
            Character::Percent =>
                CharacterInfo {
                    value: 41,
                    label: "%",
                    name: "Percent Symbol",
                    note: "Punctuation",
                },
            Character::Caret =>
                CharacterInfo { value: 42, label: "^", name: "Caret Symbol", note: "Punctuation" },
            Character::Ampersand =>
                CharacterInfo { value: 43, label: "&", name: "Ampersand", note: "Punctuation" },
            Character::Asterisk =>
                CharacterInfo { value: 44, label: "*", name: "Asterisk", note: "Punctuation" },
            Character::LeftParen =>
                CharacterInfo {
                    value: 45,
                    label: "(",
                    name: "Left Parenthesis",
                    note: "Punctuation",
                },
            Character::RightParen =>
                CharacterInfo {
                    value: 46,
                    label: ")",
                    name: "Right Parenthesis",
                    note: "Punctuation",
                },
            Character::Hyphen =>
                CharacterInfo { value: 47, label: "-", name: "Hyphen", note: "Punctuation" },
            Character::Underscore =>
                CharacterInfo { value: 48, label: "_", name: "Underscore", note: "Punctuation" },
            Character::Equals =>
                CharacterInfo { value: 49, label: "=", name: "Equals Sign", note: "Punctuation" },
            Character::Plus =>
                CharacterInfo { value: 50, label: "+", name: "Plus Sign", note: "Punctuation" },
            Character::LeftBracket =>
                CharacterInfo { value: 51, label: "[", name: "Left Bracket", note: "Punctuation" },
            Character::RightBracket =>
                CharacterInfo { value: 52, label: "]", name: "Right Bracket", note: "Punctuation" },
            Character::LeftBrace =>
                CharacterInfo { value: 53, label: "{", name: "Left Brace", note: "Punctuation" },
            Character::RightBrace =>
                CharacterInfo { value: 54, label: "}", name: "Right Brace", note: "Punctuation" },
            Character::Backslash =>
                CharacterInfo { value: 55, label: "\\", name: "Backslash", note: "Punctuation" },
            Character::Pipe =>
                CharacterInfo { value: 56, label: "|", name: "Pipe", note: "Punctuation" },
            Character::Semicolon =>
                CharacterInfo { value: 57, label: ";", name: "Semicolon", note: "Punctuation" },
            Character::Colon =>
                CharacterInfo { value: 58, label: ":", name: "Colon", note: "Punctuation" },
            Character::SingleQuote =>
                CharacterInfo { value: 59, label: "'", name: "Single Quote", note: "Punctuation" },
            Character::DoubleQuote =>
                CharacterInfo { value: 60, label: "\"", name: "Double Quote", note: "Punctuation" },
            Character::Comma =>
                CharacterInfo { value: 61, label: ",", name: "Comma", note: "Punctuation" },
            Character::Period =>
                CharacterInfo { value: 62, label: ".", name: "Period", note: "Punctuation" },
            Character::LessThan =>
                CharacterInfo { value: 63, label: "<", name: "Less Than", note: "Punctuation" },
            Character::GreaterThan =>
                CharacterInfo { value: 64, label: ">", name: "Greater Than", note: "Punctuation" },
            Character::Slash =>
                CharacterInfo { value: 65, label: "/", name: "Slash", note: "Punctuation" },
            Character::Question =>
                CharacterInfo { value: 66, label: "?", name: "Question Mark", note: "Punctuation" },
            Character::Space =>
                CharacterInfo { value: 0, label: " ", name: "Space", note: "Whitespace" },
        }
    }
}

fn main() {
    let hello_world = vec![
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 8, 5, 12, 12, 15, 0, 23, 15, 18, 12, 4, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
    ];
    let mut character_codes = HashMap::new();
    character_codes.insert(Character::A, Character::A.info());
    character_codes.insert(Character::B, Character::B.info());
    // ...insert other characters...
    character_codes.insert(Character::Space, Character::Space.info());

    // Example usage
    // println!("Info for A: {:?}", character_codes[&Character::A]);
}