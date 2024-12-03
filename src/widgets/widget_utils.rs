pub type WidgetOutput = Vec<String>;

pub fn pad_or_truncate_lines(lines: Vec<String>) -> WidgetOutput {
    let mut output = lines
        .into_iter()
        .map(|line| line.chars().take(22).collect::<String>())
        .collect::<Vec<String>>();

    // Pad with empty strings if there are less than 6 lines
    while output.len() < 6 {
        output.push(String::new());
    }

    // Truncate to ensure exactly 6 lines
    if output.len() > 6 {
        output.truncate(6);
    }

    output
}

fn center_line(line: &mut [u8; 22]) {
    let mut start = 0;
    let mut end = 21;
    while start < end && line[start] == 0 {
        start += 1;
    }
    while end > start && line[end] == 0 {
        end -= 1;
    }
    let len = end - start + 1;
    let padding = (22 - len) / 2;
    if padding > 0 {
        for i in (start..=end).rev() {
            if i + padding < 22 {
                line[i + padding] = line[i];
            }
        }
        for i in start..start + padding {
            if i < 22 {
                line[i] = 0;
            }
        }
        for i in end + padding + 1..22 {
            if i < 22 {
                line[i] = 0;
            }
        }
    }
}

fn center_message_vertically(message: &mut [[u8; 22]; 6]) {
    let vertical_padding = (6 - message.len()) / 2;
    println!("Vertical padding: {}", vertical_padding);
    if vertical_padding > 0 {
        for i in (0..6).rev() {
            if i >= vertical_padding {
                message[i] = message[i - vertical_padding];
            } else {
                message[i] = [0; 22];
            }
        }
    }
}

pub fn format_message(message: &str) -> Option<WidgetOutput> {
    let mut formatted_message: Vec<String> = Vec::new();
    let words: Vec<&str> = message.split_whitespace().collect();
    let mut current_line = String::new();

    for word in words {
        if current_line.len() + word.len() + 1 > 22 {
            formatted_message.push(current_line);
            current_line = String::new();
        }
        if !current_line.is_empty() {
            current_line.push(' ');
        }
        current_line.push_str(word);
    }

    if !current_line.is_empty() {
        formatted_message.push(current_line);
    }

    Some(pad_or_truncate_lines(formatted_message))
}
