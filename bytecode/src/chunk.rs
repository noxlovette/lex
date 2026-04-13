use strum::Display;

use crate::InterpreterError;

#[derive(Debug, Display)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
/// The OpCode of the bytecode
pub enum OpCode {
    /// Return instruction
    Return,
    /// Constants during runtime
    Constant,
}

impl TryFrom<u8> for OpCode {
    type Error = InterpreterError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Return),
            1 => Ok(Self::Constant),
            _ => Err(InterpreterError::UnknownOp),
        }
    }
}
/// A chunk of instructions
#[derive(Default)]
pub struct Chunk {
    pub(crate) code: Vec<u8>,
    pub(crate) constants: Vec<Value>,
    pub(crate) lines: Vec<usize>,
}

impl Chunk {
    /// Creates a new chunk
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds an OpCode to the chunk
    pub fn write(&mut self, b: u8, line: usize) {
        self.code.push(b);
        self.lines.push(line);
    }

    /// Adds a constant to the chunk
    pub fn add_constant(&mut self, v: Value) -> usize {
        self.constants.push(v);
        // After we add the constant, we return the index where the constant was appended so that we can locate that same constant later.
        self.constants.len() - 1
    }
}
type Value = f32;
