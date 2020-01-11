use std::rc::Rc;
use rlox_compiler::{Chunk, OpCode, Value, VM};

fn main() {
    let mut chunk = Chunk::new();

    let constant = chunk.add_constant(Value::Number(456f64)).unwrap();

    chunk.add(OpCode::Constant(constant), 123);
    chunk.add(OpCode::Return, 123);

    let mut vm = VM::new(Rc::new(chunk));
    match vm.run() {
        Ok(_) => {},
        Err(err) => eprintln!("{:?}", err),
    }
}