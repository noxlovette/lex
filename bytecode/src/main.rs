use bytecode::{Chunk, OpCode};

fn main() {
    let mut chunk = Chunk::new();
    chunk.write(OpCode::Return as u8);
    chunk.disassemble("test chunk");
}
