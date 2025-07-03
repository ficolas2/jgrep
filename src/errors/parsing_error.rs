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

    // Parsing
    pub fn multiple_colons() -> Self {
        ParsingError::new(r#"Found more than one colon. If you want to use a colon as part of a key or value, enclose it in quotes. Example: ["field:with:colons"]:"value:with:colons""#.into())
    }

    pub fn malformed_range() -> Self {
        ParsingError::new(
            "Malformed range. Valid examples: `[1:3]`, `[1:]`, `[:2]`, `[-1:]`.".into(),
        )
    }

    pub fn malformed_list() -> Self {
        ParsingError::new("Malformed list. Valid example: `[0, 1, 3]`.".into())
    }

    pub fn int_negative(n: i32) -> Self {
        ParsingError::new(format!(
            "Cannot parse int '{}', cannot be negative",
            n
        ))
    }

    pub fn special_chars_in_value() -> Self {
        ParsingError::new("Found special characters in value (`[`, `]`, `,`, or `.`). If you want to match these characters, enclose the value in quotes.".into())
    }

    pub fn unexpected_dollar() -> Self {
        ParsingError::new("Unexpected `$`. The dollar sign is only allowed at the start of the pattern to assert the root position.".into())
    }

    pub fn too_many_dots() -> Self {
        ParsingError::new("Invalid sequence: `...` is not allowed. Use `.` to separate fields or `..` to match any number of fields.".into())
    }
}

impl std::fmt::Display for ParsingError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "jgrep: Error parsing patern: {}", self.message)
    }
}

impl Error for ParsingError {}
