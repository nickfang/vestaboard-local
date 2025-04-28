pub type WidgetOutput = Vec<String>;
pub const MAX_MESSAGE_LENGTH: usize = 22;
pub const MAX_MESSAGE_HEIGHT: usize = 6;

pub fn full_justify_line(s1: String, s2: String) -> String {
    let len1 = s1.chars().count();
    let len2 = s2.chars().count();
    let mut padding = 1;
    if len1 + len2 < MAX_MESSAGE_LENGTH {
        padding = MAX_MESSAGE_LENGTH - len1 - len2;
    }
    return format!("{}{:padding$}{}", s1, "", s2);
}

pub fn split_into_lines(text: &str) -> WidgetOutput {
    let mut formatted_message: Vec<String> = Vec::new();
    let words: Vec<&str> = text.split_whitespace().collect();
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
                formatted_message.push(split_word);
                split_word = split;
            }
            continue;
        }
        if current_line.len() + word.len() + 1 > MAX_MESSAGE_LENGTH {
            // if next word doesn't fit, add to formatted_message
            formatted_message.push(current_line);
            current_line = String::new();
        }

        if !current_line.is_empty() {
            // add space between words
            current_line.push(' ');
        }
        current_line.push_str(word);
    }

    if !current_line.is_empty() {
        formatted_message.push(current_line);
    }

    formatted_message
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

pub fn center_message(mut message: Vec<String>, height: usize) -> WidgetOutput {
    if message.len() < height {
        let half_padding = (height - message.len()) / 2;
        for _ in 0..half_padding {
            message.insert(0, String::new());
        }
        while message.len() < height {
            message.push(String::new());
        }
    }
    message
}

pub fn format_message(message: &str) -> Option<WidgetOutput> {
    let mut formatted_message: Vec<String> = Vec::new();
    split_into_lines(message)
        .iter()
        .for_each(|line| {
            formatted_message.push(center_line(line.to_string()));
        });
    Some(center_message(formatted_message, MAX_MESSAGE_HEIGHT))
}

// There is only room for 4 lines of error message on the Vestaboard
pub fn format_error(error: &str) -> WidgetOutput {
    let mut formatted_message: Vec<String> = Vec::new();
    let lowercase_error = error.to_lowercase();
    let words: Vec<&str> = lowercase_error.split_whitespace().collect();
    let mut current_line = String::new();
    formatted_message.push("R R R R error: R R R R".to_string());
    formatted_message.push("".to_string());
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
    formatted_message
}
