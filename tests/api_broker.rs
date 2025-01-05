use vestaboard_local::api_broker::to_codes;

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
fn test_message_with_numbers() {
    let message = "1234567890";
    let expected_codes = Some(vec![27, 28, 29, 30, 31, 32, 33, 34, 35, 36]);
    assert_eq!(to_codes(message), expected_codes);
}

#[test]
fn test_message_with_colors() {
    let message = "ROYGBVWK";
    let expected_codes = Some(vec![63, 64, 65, 66, 67, 68, 69, 70]);
    assert_eq!(to_codes(message), expected_codes);
}
