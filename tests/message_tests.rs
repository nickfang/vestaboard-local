use vestaboard_local::message::to_codes;

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
