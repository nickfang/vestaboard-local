#[path = "../cli_display.rs"]
mod cli_display;

#[cfg(test)]
#[test]
fn print_message_test() {
    let message = vec![
        "ROYGBVKW".to_string(),
        "abcdefghijklmnopqrstuv".to_string(),
        "wxyz1234567890".to_string(),
        "!@#$()-+&=;:'\"%,./?D".to_string()
    ];

    let output = cli_display::print_message(message, "");

    assert_eq!(output, [
        "Vestaboard Display:",
        "|----------------------|",
        "|\u{1b}[31m█\u{1b}[0m\u{1b}[38:5:208m█\u{1b}[0m\u{1b}[33m█\u{1b}[0m\u{1b}[32m█\u{1b}[0m\u{1b}[34m█\u{1b}[0m\u{1b}[35m█\u{1b}[0m\u{1b}[30m█\u{1b}[0m\u{1b}[37m█\u{1b}[0m              |",
        "|abcdefghijklmnopqrstuv|",
        "|wxyz1234567890        |",
        "|!@#$()-+&=;:'\"%,./?°  |",
        "|                      |",
        "|                      |",
        "|----------------------|",
    ]);
}
#[test]
fn print_message_with_colors_test() {
    let message = vec![
        "R".to_string(),
        "O".to_string(),
        "Y".to_string(),
        "G".to_string(),
        "B".to_string(),
        "V".to_string()
    ];

    let output = cli_display::print_message(message, "");

    assert_eq!(output, [
        "Vestaboard Display:",
        "|----------------------|",
        "|\u{1b}[31m█\u{1b}[0m                     |",
        "|\u{1b}[38:5:208m█\u{1b}[0m                     |",
        "|\u{1b}[33m█\u{1b}[0m                     |",
        "|\u{1b}[32m█\u{1b}[0m                     |",
        "|\u{1b}[34m█\u{1b}[0m                     |",
        "|\u{1b}[35m█\u{1b}[0m                     |",
        "|----------------------|",
    ]);
}

#[test]
fn print_message_with_degree_character() {
    let message = vec![
        "100D is really hot".to_string(),
        "0D is really cold".to_string(),
        "but even worse".to_string(),
        "in metric.".to_string()
    ];
    let output = cli_display::print_message(message, "");
    assert_eq!(output, [
        "Vestaboard Display:",
        "|----------------------|",
        "|100° is really hot    |",
        "|0° is really cold     |",
        "|but even worse        |",
        "|in metric.            |",
        "|                      |",
        "|                      |",
        "|----------------------|",
    ])
}

#[test]
fn print_message_with_title() {
    let message = vec![
        "".to_string(),
        "".to_string(),
        "     don't panic!     ".to_string(),
        "".to_string()
    ];
    let output = cli_display::print_message(message, "Hitchhiker's Guide to the Galaxy");
    assert_eq!(output, [
        "Hitchhiker's Guide to the Galaxy",
        "|----------------------|",
        "|                      |",
        "|                      |",
        "|     don't panic!     |",
        "|                      |",
        "|                      |",
        "|                      |",
        "|----------------------|",
    ])
}
