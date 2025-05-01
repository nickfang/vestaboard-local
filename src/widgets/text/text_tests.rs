use std::path;

use super::text::{ get_text, get_text_from_file };

#[cfg(test)]
#[test]
fn test_text_get_text() {
    let text = get_text("Hello, world!");
    assert_eq!(text, ["", "", "    Hello, world!     ", "", "", ""]);
}

#[test]
fn test_text_get_text_from_file() {
    let path = std::path::PathBuf::from("text.txt");
    let text = get_text_from_file(path);
    println!("{:?}", text);
    // assert_eq!(text, text);
}
