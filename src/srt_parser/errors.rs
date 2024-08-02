use std::fmt;

#[derive(Debug)]
pub enum ParsingError {
    MalformedSubtitle,
    InvalidTimestamp,
    InvalidNumber,
    IoError(std::io::Error),
}

impl From<std::io::Error> for ParsingError {
    fn from(error: std::io::Error) -> Self {
        ParsingError::IoError(error)
    }
}

impl fmt::Display for ParsingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParsingError::MalformedSubtitle => write!(f, "Malformed subtitle"),
            ParsingError::InvalidTimestamp => write!(f, "Invalid timestamp"),
            ParsingError::InvalidNumber => write!(f, "Invalid subtitle number"),
            ParsingError::IoError(e) => write!(f, "I/O error: {}", e),
        }
    }
}
