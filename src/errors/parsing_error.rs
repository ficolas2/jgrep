use std::error::Error;

#[derive(Debug, Eq, PartialEq)]
pub struct ParsingError {
    message: String,
}

impl ParsingError {
    pub fn new(message: String) -> Self {
        ParsingError { message }
    }

    // Tokenization
    pub fn missmatched_brackets(position: usize) -> Self {
        Self::new(format!(
            "Invalid pattern: missmatched brackets. Opening [ at position {} had no closing ]",
            position
        ))
    }

    pub fn missmatched_quotes(position: usize) -> Self {
        ParsingError::new(format!(
            r#"Invalid pattern: missmatched quotes. Opening " at position {} had no closing ""#,
            position
        ))
    }

    pub fn integer_is_too_big(number_str: &str) -> Self {
        ParsingError::new(format!("Integer is too big: {}", number_str))
    }
}

impl std::fmt::Display for ParsingError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "jgrep: Error parsing patern: {}", self.message)
    }
}

impl Error for ParsingError {}
