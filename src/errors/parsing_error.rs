use std::error::Error;


#[derive(Debug)]
pub struct ParsingError {
    message: String,
}

impl ParsingError {
    pub fn new(message: String) -> Self {
        ParsingError { message }
    }
}

impl std::fmt::Display for ParsingError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "jgrep: Error parsing patern: {}", self.message)
    }
}

impl Error for ParsingError {}
