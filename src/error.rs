use std::io;
use thiserror::Error;

pub type InterpreterResult<T> = Result<T, InterpreterError>;

#[derive(Debug, Error)]
pub enum InterpreterError {
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
}
