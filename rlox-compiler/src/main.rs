use rlox_compiler::{Chunk, OpCode, Value, disasm};

fn main() {
    let stdout = std::io::stdout();
    let mut handle = stdout.lock();

    let mut chunk = Chunk::new();

    let constant = chunk.add_constant(Value::Number(456f64)).unwrap();

    chunk.add(OpCode::Constant(constant), 123);
    chunk.add(OpCode::Return, 123);

    disasm(&mut handle, &chunk, "main").unwrap();
}