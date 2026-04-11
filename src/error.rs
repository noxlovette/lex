use crate::{Token, Value};
use std::io;
use thiserror::Error;

pub(crate) type CompiletimeResult<T> = Result<T, CompiletimeError>;
pub(crate) type RuntimeResult<T> = Result<T, RuntimeError>;
pub(crate) type EvalResult<T> = Result<T, RuntimeControl>;

#[derive(Debug)]
pub(crate) enum RuntimeControl {
    Error(RuntimeError),
    Return(Value),
}

impl From<RuntimeControl> for RuntimeError {
    fn from(value: RuntimeControl) -> Self {
        match value {
            RuntimeControl::Error(err) => err,
            RuntimeControl::Return(_) => RuntimeError::ReturnOutsideFunction,
        }
    }
}

impl<T> From<RuntimeError> for EvalResult<T> {
    fn from(value: RuntimeError) -> Self {
        Self::Err(RuntimeControl::Error(value))
    }
}

impl From<RuntimeError> for RuntimeControl {
    fn from(value: RuntimeError) -> Self {
        RuntimeControl::Error(value)
    }
}

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
    #[error("Cannot read local variable in its own initialiser: {0}")]
    InitializerError(Token),
}

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("Type Error: {message:?}. Found: {value:?}")]
    TypeError { message: String, value: Value },
    #[error("Undefined variable '{lexeme:?}'")]
    Undefined { lexeme: String },
    #[error("Expression not callable {0}")]
    NotCallable(String),
    #[error("Expected {expected:?} arguments but got {got:?}")]
    Arity { expected: usize, got: usize },
    #[error("Can't return from top-level code.")]
    ReturnOutsideFunction,
}
