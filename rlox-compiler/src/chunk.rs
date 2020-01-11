use std::collections::HashMap;
use crate::op::{ OpCode, DecodeError };
use crate::Value;

pub struct Chunk {
    code: Vec<u8>,
    lines: HashMap<usize, usize>,
    constants: Vec<Value>,
}

impl Chunk {
    pub fn new() -> Chunk {
        Chunk {
            code: Vec::new(),
            lines: HashMap::new(),
            constants: Vec::new(),
        }
    }

    pub fn add_constant(&mut self, value: Value) -> Result<u8, String> {
        if self.constants.len() >= 255 {
            Err(String::from("too many local constants"))
        } else {
            self.constants.push(value);
            Ok((self.constants.len() - 1) as u8)
        }
    }

    pub fn add(&mut self, op: OpCode, line: usize) {
        let mut bytes = op.encode();
        self.lines.insert(self.code.len(), line);
        self.code.append(&mut bytes);
    }

    pub fn decode(&self, offset: usize) -> Result<(OpCode, usize), DecodeError> {
        if offset >= self.code.len() {
            Err(DecodeError::EOF)
        } else {
            OpCode::decode(&self.code[offset..])
        }
    }
    pub fn as_bytes(&self) -> ::std::slice::Iter<u8> {
        self.code.iter()
    }
    pub fn constant(&self, index: u8) -> Result<&Value, String> {
        let len = self.constants.len() as u8;
        if index >= len {
            Err(format!("invalid constant index {} of {}", index, len))
        } else {
            Ok(&self.constants[index as usize])
        }
    }
    pub fn line(&self, mut offset: usize) -> usize {
        while offset > 0 {
            if let Some(&line) = self.lines.get(&offset) {
                return line;
            }

            offset -= 1;
        }

        *self.lines.get(&0).unwrap_or(&0)
    }
}