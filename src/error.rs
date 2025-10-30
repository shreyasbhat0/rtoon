use thiserror::Error;

pub type ToonResult<T> = std::result::Result<T, ToonError>;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum ToonError {
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Parse error at line {line}, column {column}: {message}")]
    ParseError {
        line: usize,
        column: usize,
        message: String,
    },

    #[error("Invalid character '{char}' at position {position}")]
    InvalidCharacter { char: char, position: usize },

    #[error("Unexpected end of input")]
    UnexpectedEof,

    #[error("Type mismatch: expected {expected}, found {found}")]
    TypeMismatch { expected: String, found: String },

    #[error("Invalid delimiter: {0}")]
    InvalidDelimiter(String),

    #[error("Array length mismatch: expected {expected}, found {found}")]
    LengthMismatch { expected: usize, found: usize },

    #[error("Invalid structure: {0}")]
    InvalidStructure(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Deserialization error: {0}")]
    DeserializationError(String),
}

impl ToonError {
    pub fn parse_error(line: usize, column: usize, message: impl Into<String>) -> Self {
        ToonError::ParseError {
            line,
            column,
            message: message.into(),
        }
    }

    pub fn invalid_char(char: char, position: usize) -> Self {
        ToonError::InvalidCharacter { char, position }
    }

    pub fn type_mismatch(expected: impl Into<String>, found: impl Into<String>) -> Self {
        ToonError::TypeMismatch {
            expected: expected.into(),
            found: found.into(),
        }
    }

    pub fn length_mismatch(expected: usize, found: usize) -> Self {
        ToonError::LengthMismatch { expected, found }
    }
}
