use std::collections::HashMap;
use std::rc::Rc;
use crate::op::{ OpCode, DecodeError };
use crate::Value;
use crate::disasm::disassemble_instruction;

pub struct Chunk {
    code: Vec<u8>,
    lines: HashMap<usize, usize>,
    constants: Vec<Rc<Value>>,
}

pub struct ChunkReference {
    offset: usize,
    length: usize,
}

impl Chunk {
    pub fn new() -> Chunk {
        Chunk {
            code: Vec::new(),
            lines: HashMap::new(),
            constants: Vec::new(),
        }
    }

    pub fn len(&self) -> usize { self.code.len() }

    pub fn add_constant(&mut self, value: Value) -> Result<u8, String> {
        if self.constants.len() >= 255 {
            Err(String::from("too many local constants"))
        } else {
            self.constants.push(Rc::new(value));
            Ok((self.constants.len() - 1) as u8)
        }
    }

    pub fn add(&mut self, op: OpCode, line: usize) -> ChunkReference {
        let mut bytes = op.encode();

        let offset = self.code.len();
        let length = bytes.len();

        self.lines.insert(offset, line);
        self.code.append(&mut bytes);

        ChunkReference { offset, length }
    }

    pub fn patch(&mut self, location: &ChunkReference, op: OpCode) {
        let patch_bytes = op.encode();
        if patch_bytes.len() != location.length {
            panic!("Attempting to patch {} bytes into a {} byte location", patch_bytes.len(), location.length);
        }

        for (i, b) in patch_bytes.iter().enumerate() {
            self.code[location.offset + i] = *b;
        }
    }

    pub fn decode(&self, offset: usize) -> Result<(OpCode, usize), DecodeError> {
        if offset >= self.code.len() {
            Err(DecodeError::EOF)
        } else {
            let (op, len) = OpCode::decode(&self.code[offset..])?;
            Ok((op, offset + len))
        }
    }
    pub fn as_bytes(&self) -> ::std::slice::Iter<u8> {
        self.code.iter()
    }
    pub fn constant(&self, index: u8) -> Result<Rc<Value>, String> {
        let len = self.constants.len() as u8;
        if index >= len {
            Err(format!("invalid constant index {} of {}", index, len))
        } else {
            Ok(Rc::clone(&self.constants[index as usize]))
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