use crate::Error;
use strum::Display;

#[derive(Debug, Display)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum OpCode {
    Return,
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
}

impl Chunk {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn write(&mut self, b: u8) {
        self.code.push(b);
    }
}
