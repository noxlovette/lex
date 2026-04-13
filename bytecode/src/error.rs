use thiserror::Error;

pub type InterpreterResult = Result<(), InterpreterError>;

/// Bytecode Error
#[derive(Debug, Error)]
pub enum InterpreterError {
    #[error("Unknown Op")]
    /// Unknown Operation
    UnknownOp,
}
pub enum CompileError {}
pub enum RuntimeError {}
