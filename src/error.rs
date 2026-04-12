use crate::{Token, Value};
use std::io;
use thiserror::Error;

pub(crate) type CompiletimeResult<T> = Result<T, CompiletimeError>;
pub(crate) type RuntimeResult<T> = Result<T, RuntimeError>;
pub(crate) type EvalResult<T> = Result<T, RuntimeControl>;

#[derive(Debug)]
pub(crate) enum RuntimeControl {
    Error(Box<RuntimeError>),
    Return(Box<Value>),
}

impl From<RuntimeControl> for RuntimeError {
    fn from(value: RuntimeControl) -> Self {
        match value {
            RuntimeControl::Error(err) => *err,
            RuntimeControl::Return(_) => RuntimeError::ReturnOutsideFunction,
        }
    }
}

impl<T> From<RuntimeError> for EvalResult<T> {
    fn from(value: RuntimeError) -> Self {
        Self::Err(RuntimeControl::Error(Box::new(value)))
    }
}

impl From<RuntimeError> for RuntimeControl {
    fn from(value: RuntimeError) -> Self {
        RuntimeControl::Error(Box::new(value))
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
    #[error("Already a variable with this name in this scope: {0}")]
    AlreadyDeclared(Token),
    #[error("Can't return from top-level code.")]
    ReturnOutsideFunction,
    #[error("Can't return a value from an initializer.")]
    ReturnFromInitializer,
    #[error("Can't use 'this' outside of a class.")]
    ThisOutsideClass,
    #[error("Can't use 'super' outside of a class.")]
    SuperOutsideClass,
    #[error("Can't use 'super' in a class with no superclass.")]
    SuperWithoutSuperclass,
    #[error("A class can't inherit from itself.")]
    InheritSelf(Token),
}

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("Type Error: {message:?}. Found: {value:?}")]
    TypeError { message: String, value: Box<Value> },
    #[error("Undefined variable {lexeme:?}")]
    Undefined { lexeme: String },
    #[error("Expression not callable {0}")]
    NotCallable(String),
    #[error("Expected {expected:?} arguments but got {got:?}")]
    Arity { expected: usize, got: usize },
    #[error("Can't return from top-level code.")]
    ReturnOutsideFunction,
    #[error("Only instances have fields.")]
    FieldsOnInstancesOnly,
    #[error("Only instances have properties.")]
    PropertiesOnInstancesOnly,
    #[error("Superclass must be a class.")]
    SuperclassMustBeClass,
}
