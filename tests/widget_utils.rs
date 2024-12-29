use vestaboard_local::widgets::widget_utils::{ format_message, center_line };

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
    let expected = vec![
        "".to_string(),
        "".to_string(),
        "     hello world      ".to_string(),
        "".to_string(),
        "".to_string(),
        "".to_string()
    ];
    // let expected = [
    //     [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    //     [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    //     [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    //     [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    //     [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    //     [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    // ];
    assert_eq!(formatted, expected);
}

#[test]
fn test_format_message_long_word() {
    let message = "thisisaverylongwordthatshouldwrap";
    let formatted = format_message(message).unwrap();
    let expected = vec![
        "".to_string(),
        "".to_string(),
        "thisisaverylongword".to_string(),
        "thatshouldwrap".to_string(),
        "".to_string(),
        "".to_string()
    ];
    // let expected = [
    //     [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    //     [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    //     [20, 8, 9, 19, 9, 19, 1, 22, 5, 18, 25, 12, 15, 14, 7, 23, 15, 18, 4, 20, 8, 1],
    //     [0, 0, 0, 0, 0, 20, 19, 8, 15, 21, 12, 4, 23, 18, 1, 16, 0, 0, 0, 0, 0, 0],
    //     [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    //     [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    // ];
    assert_eq!(formatted, expected);
}

#[test]
fn test_format_message_colors() {
    let message = "ROYGBVWKF";
    let formatted = format_message(message).unwrap();
    let expected = vec![
        "".to_string(),
        "".to_string(),
        "      ROYGBVWKF       ".to_string(),
        "".to_string(),
        "".to_string(),
        "".to_string()
    ];
    // let expected = [
    //     [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    //     [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    //     [0, 0, 0, 0, 0, 0, 63, 64, 65, 66, 67, 68, 69, 70, 71, 0, 0, 0, 0, 0, 0, 0],
    //     [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    //     [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    //     [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    // ];
    assert_eq!(formatted, expected);
}
