pub type WidgetOutput = Vec<String>;

fn center_line(line: String) -> String {
    let half_padding = (22 - line.len()) / 2;
    println!("Half padding: {}", half_padding);
    if half_padding > 0 {
        format!("{}{}", " ".repeat(half_padding), line)
    } else {
        line
    }
}

fn center_message(mut message: Vec<String>) -> WidgetOutput {
    if message.len() < 6 {
        let half_padding = (6 - message.len()) / 2;
        for _ in 0..half_padding {
            message.insert(0, String::new());
        }
        while message.len() < 6 {
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
        if current_line.len() + word.len() + 1 > 22 {
            let padded_line = center_line(current_line);
            println!("Padded line: {}", padded_line);
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

    Some(center_message(formatted_message))
}

pub fn format_error(error: &str) -> WidgetOutput {
    let mut formatted_message: Vec<String> = Vec::new();
    let words: Vec<&str> = error.split_whitespace().collect();
    let mut current_line = String::new();

    for word in words {
        if current_line.len() + word.len() + 1 > 22 {
            let padded_line = center_line(current_line);
            println!("Padded line: {}", padded_line);
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
