use crate::op::OpCode;

pub struct Chunk {
    code: Vec<OpCode>,
}

impl Chunk {
    pub fn new() -> Chunk {
        Chunk {
            code: Vec::new(),
        }
    }

    pub fn add(&mut self, op: OpCode) {
        self.code.push(op);
    }
}