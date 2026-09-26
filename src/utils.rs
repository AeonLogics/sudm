use std::io;
use std::io::Write;
use colored::Colorize;

pub fn confirm(prompt: &str) -> bool {
    print!("{} [y/N]: ", prompt.yellow());
    let _ = io::stdout().flush();
    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_err() {
        return false;
    }
    matches!(input.trim().to_lowercase().as_str(), "y" | "yes")
}

pub fn is_valid_identifier(name: &str) -> bool {
    !name.is_empty()
        && name.chars().next().unwrap().is_ascii_alphabetic()
        && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}