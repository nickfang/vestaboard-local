#[path = "../widget_utils.rs"]
mod widget_utils;
use widget_utils::{ format_message, format_error, center_line, full_justify_line };

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

#[test]
fn test_format_error() {
    let message = "This is an error message to display on the Vestaboard.";
    let formatted = format_error(message);
    let expected = vec![
        "R R R R error: R R R R".to_string(),
        "".to_string(),
         "   this is an error   ".to_string(),
         "message to display on ".to_string(),
         "   the vestaboard.    ".to_string(),
    ];
    assert_eq!(formatted, expected);
}

#[test]
fn test_full_justify_line() {
    let s1 = "hello".to_string();
    let s2 = "world".to_string();
    let justified = full_justify_line(s1, s2);
    let expected = "hello            world".to_string();
    assert_eq!(justified, expected);
    assert_eq!(expected.chars().count(), 22);
}

#[test]
fn test_full_justify_line_long_words() {
    let longs1 = "thisisaverylongword".to_string();
    let longs2 = "thatshouldwrap".to_string();
    let justified = full_justify_line(longs1, longs2);
    let expected = "thisisaverylongword thatshouldwrap";
    assert_eq!(justified, expected);
}

#[test]
fn test_full_justify_line_empty_strings() {
    let emptys1 = "".to_string();
    let s2 = "world".to_string();
    let justified = full_justify_line(emptys1, s2);
    let expected = "                 world".to_string();
    assert_eq!(justified, expected);
    assert_eq!(expected.chars().count(), 22);

    let s1 = "hello".to_string();
    let emptys2 = "".to_string();
    let justified = full_justify_line(s1, emptys2);
    let expected = "hello                 ".to_string();
    assert_eq!(justified, expected);
    assert_eq!(expected.chars().count(), 22);
}
