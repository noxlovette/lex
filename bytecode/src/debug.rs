use crate::{Chunk, OpCode};

impl Chunk {
    #[cfg(debug_assertions)]
    pub fn disassemble(&self, name: &str) {
        println!("== {name} ==");
        let mut offset = 0;
        while offset < self.code.len() {
            offset = self.disassemble_instruction(offset)
        }
    }

    fn disassemble_instruction(&self, offset: usize) -> usize {
        print!("{:04} ", offset);
        match self.code[offset].try_into() {
            Ok(op) => Self::simple_instruction(&op, offset),
            Err(e) => {
                eprintln!("{e}");
                offset + 1
            }
        }
    }

    fn simple_instruction(code: &OpCode, offset: usize) -> usize {
        println!("{code}");
        offset + 1
    }
}
