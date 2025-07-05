#[cfg(test)]
mod tests {
  use crate::widgets::jokes::get_joke;

  #[test]
  fn test_get_joke_success() {
    let result = get_joke();
    assert!(result.is_ok(), "get_joke should return Ok result");

    let lines = result.unwrap();
    assert!(!lines.is_empty(), "Joke should produce some output");

    // The joke should be formatted properly (6 lines for Vestaboard)
    assert_eq!(
      lines.len(),
      6,
      "Joke should be formatted to 6 lines for Vestaboard"
    );

    // Check that the joke content is present somewhere in the output
    let combined_text = lines.join("");
    assert!(
      combined_text.to_lowercase().contains("supplies"),
      "Joke should contain the punchline 'supplies'"
    );
  }

  #[test]
  fn test_get_joke_consistency() {
    // The joke should be consistent across calls
    let result1 = get_joke();
    let result2 = get_joke();

    assert!(result1.is_ok(), "First call should succeed");
    assert!(result2.is_ok(), "Second call should succeed");

    let lines1 = result1.unwrap();
    let lines2 = result2.unwrap();

    assert_eq!(lines1, lines2, "Joke should be consistent across calls");
  }

  #[test]
  fn test_get_joke_format() {
    let result = get_joke();
    assert!(result.is_ok(), "get_joke should return Ok result");

    let lines = result.unwrap();

    // Each line should be valid for Vestaboard (22 characters max)
    for line in &lines {
      assert!(
        line.len() <= 22,
        "Each line should be 22 characters or less, found: '{}' (length: {})",
        line,
        line.len()
      );
    }

    // Lines should only contain valid Vestaboard characters
    for line in &lines {
      for ch in line.chars() {
        assert!(
          ch.is_ascii()
            && (ch.is_alphanumeric() || ch.is_whitespace() || "!\"#$%&'()*+,-./:;?@".contains(ch)),
          "Invalid character '{}' found in line '{}'",
          ch,
          line
        );
      }
    }
  }
}
