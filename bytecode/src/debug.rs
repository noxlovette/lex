use crate::{Chunk, OpCode};

impl Chunk {
    #[cfg(debug_assertions)]
    /// Convert an instruction into a human-friendly format for debugging
    pub fn disassemble(&self, name: &str) {
        println!("== {name} ==");
        let mut offset = 0;
        while offset < self.code.len() {
            offset = self.disassemble_instruction(offset)
        }
    }

    fn disassemble_instruction(&self, offset: usize) -> usize {
        use OpCode::*;
        print!("{:04} ", offset);
        if offset > 0 && self.lines[offset] == self.lines[offset - 1] {
            print!("   | ");
        } else {
            print!("{:4} ", self.lines[offset]);
        }

        match self.code[offset].try_into() {
            Ok(op) => match op {
                Return => Self::simple_instruction(&op, offset),
                Constant => self.constant_instruction(&op, offset),
            },
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

    fn constant_instruction(&self, code: &OpCode, offset: usize) -> usize {
        let constant = self.code[offset + 1];
        print!("{:<16} {:>4} '", code, constant);
        println!("{}", self.constants[constant as usize]);
        offset + 2
    }
}
