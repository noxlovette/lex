use crate::Error;
use strum::Display;

#[derive(Debug, Display)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum OpCode {
    Return,
    Constant,
}

impl TryFrom<u8> for OpCode {
    type Error = Error;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Return),
            _ => Err(Error::Op),
        }
    }
}
#[derive(Default)]
pub struct Chunk {
    pub(crate) code: Vec<u8>,
    pub(crate) constants: Vec<Value>,
}

impl Chunk {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn write(&mut self, b: u8) {
        self.code.push(b);
    }

    pub fn add_constant(&mut self, v: Value) -> usize {
        self.constants.push(v);
        // After we add the constant, we return the index where the constant was appended so that we can locate that same constant later.
        self.constants.len() - 1
    }
}
type Value = f32;
