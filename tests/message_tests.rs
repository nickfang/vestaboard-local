use vestaboard_local::message::{ to_codes, format_message };

#[test]
fn test_valid_message() {
    let message = "hello";
    let expected_codes = Some(vec![8, 5, 12, 12, 15]);
    assert_eq!(to_codes(message), expected_codes);
}

#[test]
fn test_invalid_message() {
    let message = "Hello!";
    assert_eq!(to_codes(message), None);
}

#[test]
fn test_empty_message() {
    let message = "";
    let expected_codes: Option<Vec<u8>> = Some(vec![]);
    assert_eq!(to_codes(message), expected_codes);
}

#[test]
fn test_message_with_spaces() {
    let message = "hello world";
    let expected_codes = Some(vec![8, 5, 12, 12, 15, 0, 23, 15, 18, 12, 4]);
    assert_eq!(to_codes(message), expected_codes);
}

#[test]
fn test_format_message_centered() {
    let message = "hello world";
    let formatted = format_message(message).unwrap();
    let expected = vec![
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 8, 5, 12, 12, 15, 0, 23, 15, 18, 12, 4, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
    ];
    assert_eq!(formatted, expected);
}

#[test]
fn test_format_message_long_word() {
    let message = "thisisaverylongwordthatshouldwrap";
    let formatted = format_message(message).unwrap();
    let expected = vec![
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [20, 8, 9, 19, 9, 19, 1, 22, 5, 18, 25, 12, 15, 14, 7, 23, 15, 18, 4, 20, 8, 1],
        [0, 0, 0, 0, 0, 20, 19, 8, 15, 21, 12, 4, 23, 18, 1, 16, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
    ];
    assert_eq!(formatted, expected);
}

#[test]
fn test_format_message_colors() {
    let message = "ROYGBVWKF";
    let formatted = format_message(message).unwrap();
    let expected = vec![
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 63, 64, 65, 66, 67, 68, 69, 70, 71, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
    ];
    assert_eq!(formatted, expected);
}
