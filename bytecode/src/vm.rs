use crate::{Chunk, InterpreterResult};

/// The actual machine that reads bytecode
pub struct Vm<'a> {
    /// Points to the chunk being read
    chunk: &'a Chunk,
    /// The index of the next instruction to be executed inside the chunk
    ///
    /// It's faster to dereference a pointer than to look up stuff in an array by index
    /// but I don't know how to do that
    ip: usize,
}

impl<'a> Vm<'a> {
    /// Creates a new VM and plugs in the first chunk
    pub fn new(chunk: &'a Chunk) -> Self {
        Self { chunk, ip: 0 }
    }

    /// Interprets a chunk, one instruction at a time
    pub fn interpret(&mut self, c: &'a Chunk) -> InterpreterResult {
        self.chunk = c;
        self.ip = 0;
        self.run()
    }

    /// The heart of the VM
    fn run(&mut self) -> InterpreterResult {
        use crate::OpCode::*;

        loop {
            #[cfg(feature = "trace_execution")]
            {
                self.chunk.disassemble_instruction(self.ip)
            }
            // Dispatch/decode the instruction
            let instruction = self.chunk.code[self.ip];
            // Point to the next byte of code
            self.ip += 1;
            match instruction.try_into()? {
                Return => break,
                Constant => {
                    let constant = self.chunk.constants[self.ip];
                    self.ip += 1;
                    println!("{:>4}", constant);

                    break;
                }
                _ => unimplemented!(),
            }
        }

        Ok(())
    }
}
