#[path = "../widget_utils.rs"]
mod widget_utils;
use widget_utils::{ format_message, center_line };

#[cfg(test)]
#[test]
fn test_center_line() {
    let line = "hello world".to_string();
    let centered = center_line(line);
    let expected = "     hello world      ";
    assert_eq!(centered, expected);
}

#[test]
fn test_format_message_centered() {
    let message = "hello world";
    let formatted = format_message(message).unwrap();
    let expected = vec!["", "", "     hello world      ", "", "", ""];
    assert_eq!(formatted, expected);
}

#[test]
fn test_format_message_long_word() {
    let message = "thisisaverylongwordthatshouldwrap";
    let formatted = format_message(message).unwrap();
    let expected = vec!["", "", "thisisaverylongwordtha", "     tshouldwrap      ", "", ""];
    assert_eq!(formatted, expected);
}

#[test]
fn test_format_message_long_word_2() {
    let message = "1 1234567890123456789012 12345678901234567890123 1234567890 12345";
    let formatted = format_message(message).unwrap();
    let expected = vec![
        "          1           ",
        "1234567890123456789012",
        "          3           ",
        "1234567890123456789012",
        "   1234567890 12345   ",
        ""
    ];
    assert_eq!(formatted, expected);
}

#[test]
fn test_format_message_colors() {
    let message = "ROYGBVWKF";
    let formatted = format_message(message).unwrap();
    let expected = vec!["", "", "      ROYGBVWKF       ", "", "", ""];
    assert_eq!(formatted, expected);
}

#[test]
fn test_format_message_full_colors() {
    let message =
        "ROYGBVWKROYGBVWKROYGBVWKROYGBVWKROYGBVWKROYGBVWKROYGBVWKROYGBVWKROYGBVWKROYGBVWKROYGBVWKROYGBVWKROYGBVWKROYGBVWKROYGBVWKROYGBVWKROYG";
    let formatted = format_message(message).unwrap();
    let expected = vec![
        "ROYGBVWKROYGBVWKROYGBV",
        "WKROYGBVWKROYGBVWKROYG",
        "BVWKROYGBVWKROYGBVWKRO",
        "YGBVWKROYGBVWKROYGBVWK",
        "ROYGBVWKROYGBVWKROYGBV",
        "WKROYGBVWKROYGBVWKROYG"
    ];
    assert_eq!(formatted, expected);
}
