use std::io;
use thiserror::Error;

use crate::Value;

pub type CompiletimeResult<T> = Result<T, CompiletimeError>;
pub type RuntimeResult<T> = Result<T, RuntimeError>;

#[derive(Debug, Error)]
pub enum CompiletimeError {
    #[error("IO Error")]
    IOError(#[from] io::Error),
    #[error("[line {line:?}] Error {location:?}: {message:?}")]
    ScannerError {
        line: usize,
        location: String,
        message: String,
    },
    #[error("Unexpected character at line {line:?}")]
    TokenError { line: usize },
    #[error("Unterminated String {line:?}")]
    StringError { line: usize },
    #[error("Number parsing error {line:?}")]
    NumberError { line: usize },
    #[error("[line {line:?}] Parsing error. {message:?}, found {lexeme:?}")]
    ParseError {
        line: usize,
        message: String,
        lexeme: String,
    },
}

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("Type Error: {message:?}. Found: {value:?}")]
    TypeError { message: String, value: Value },
}
