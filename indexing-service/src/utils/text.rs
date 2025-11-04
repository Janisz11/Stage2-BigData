use regex::Regex;
use std::collections::HashSet;

pub fn tokenize_text(text: &str) -> HashSet<String> {
    let re = Regex::new(r"\b[a-zA-Z]+\b").unwrap();
    re.find_iter(&text.to_lowercase())
        .map(|m| m.as_str().to_string())
        .filter(|word| word.len() > 2)
        .collect()
}