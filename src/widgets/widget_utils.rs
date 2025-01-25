pub type WidgetOutput = Vec<String>;
pub const MAX_MESSAGE_LENGTH: usize = 22;
pub const MAX_MESSAGE_HEIGHT: usize = 6;

pub fn full_justify_line(s1: String, s2: String) -> String {
    let len1 = s1.len();
    let len2 = s2.len();
    let mut padding = 1;
    if len1 + len2 < MAX_MESSAGE_LENGTH {
        padding = MAX_MESSAGE_LENGTH - len1 - len2;
    }

    return format!("{}{:padding$}{}", s1, "", s2);
}

pub fn center_line(line: String) -> String {
    format!("{:^1$}", line, MAX_MESSAGE_LENGTH)
    // let half_padding = (22 - line.len()) / 2;
    // if half_padding > 0 {
    //     format!("{}{}", " ".repeat(half_padding), line)
    // } else {
    //     line
    // }
}

fn center_message(mut message: Vec<String>) -> WidgetOutput {
    if message.len() < MAX_MESSAGE_HEIGHT {
        let half_padding = (MAX_MESSAGE_HEIGHT - message.len()) / 2;
        for _ in 0..half_padding {
            message.insert(0, String::new());
        }
        while message.len() < MAX_MESSAGE_HEIGHT {
            message.push(String::new());
        }
    }
    message
}

pub fn format_message(message: &str) -> Option<WidgetOutput> {
    let mut formatted_message: Vec<String> = Vec::new();
    let words: Vec<&str> = message.split_whitespace().collect();
    let mut current_line = String::new();

    for word in words {
        if word.len() > MAX_MESSAGE_LENGTH {
            let mut split_word = word.to_string();
            while !split_word.is_empty() {
                let split_index = split_word
                    .char_indices()
                    .nth(MAX_MESSAGE_LENGTH)
                    .map(|(i, _)| i)
                    .unwrap_or(split_word.len());
                let split = split_word.split_off(split_index);
                formatted_message.push(center_line(split_word));
                split_word = split;
            }
            continue;
        }
        if current_line.len() + word.len() + 1 > MAX_MESSAGE_LENGTH {
            // if next word doesn't fit, center line and add to formatted_message
            let centered_line = center_line(current_line);
            formatted_message.push(centered_line);
            current_line = String::new();
        }

        if !current_line.is_empty() {
            // add space between words
            current_line.push(' ');
        }
        current_line.push_str(word);
    }

    if !current_line.is_empty() {
        formatted_message.push(center_line(current_line));
    }

    Some(center_message(formatted_message))
}

// TODO: Not sure if should be done here.
#[allow(dead_code)]
pub fn format_error(error: &str) -> WidgetOutput {
    let mut formatted_message: Vec<String> = Vec::new();
    let words: Vec<&str> = error.split_whitespace().collect();
    let mut current_line = String::new();

    for word in words {
        if current_line.len() + word.len() + 1 > MAX_MESSAGE_LENGTH {
            let padded_line = center_line(current_line);
            formatted_message.push(padded_line);
            current_line = String::new();
        }
        if !current_line.is_empty() {
            current_line.push(' ');
        }
        current_line.push_str(word);
    }

    if !current_line.is_empty() {
        formatted_message.push(center_line(current_line));
    }

    center_message(formatted_message)
}
