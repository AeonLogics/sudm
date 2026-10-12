pub fn is_valid_identifier(name: &str) -> bool {
    !name.is_empty()
        && name.chars().next().unwrap().is_ascii_alphabetic()
        && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

#[macro_export]
macro_rules! success {
    ($($arg:tt)*) => {
        println!("{} {}", "[SUCCESS]".green().bold(), format!($($arg)*))
    };
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        println!("{} {}", "[ERROR]".red().bold(), format!($($arg)*))
    };
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        println!("{} {}", "[WARNING]".yellow().bold(), format!($($arg)*))
    };
}