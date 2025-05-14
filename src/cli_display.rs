pub fn print_message(message: Vec<String>, title: &str) -> Vec<String> {
    let mut output = Vec::new();
    if title == "" {
        output.push("Vestaboard Display:".to_string());
    } else {
        output.push(format!("{}", title));
    }
    output.push("|----------------------|".to_string());
    message
        .iter()
        .take(6)
        .for_each(|line| {
            let padded_line = format!("{:<22}", line);
            const SOLID_SQUARE: char = '\u{2588}';
            let modified_line = padded_line
                .chars()
                .map(|c| {
                    match c {
                        'D' => "°".to_string(),
                        'R' => format!("\x1b[{}m{}\x1b[0m", "31", SOLID_SQUARE),
                        'O' => format!("\x1b[{}m{}\x1b[0m", "38:5:208", SOLID_SQUARE),
                        'Y' => format!("\x1b[{}m{}\x1b[0m", "33", SOLID_SQUARE),
                        'G' => format!("\x1b[{}m{}\x1b[0m", "32", SOLID_SQUARE),
                        'B' => format!("\x1b[{}m{}\x1b[0m", "34", SOLID_SQUARE),
                        'V' => format!("\x1b[{}m{}\x1b[0m", "35", SOLID_SQUARE),
                        'W' => format!("\x1b[{}m{}\x1b[0m", "37", SOLID_SQUARE),
                        'K' => format!("\x1b[{}m{}\x1b[0m", "30", SOLID_SQUARE),
                        _ => c.to_string(),
                    }
                })
                .collect::<String>();
            output.push(format!("|{}|", modified_line));
        });
    // Make sure display matches the Vestaboard.
    // Handle if the messag is less than 6 lines.
    while output.len() < 8 {
        output.push("|                      |".to_string());
    }
    output.push("|----------------------|".to_string());
    output.iter().for_each(|line| println!("{}", line));
    output
}
