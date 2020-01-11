use std::rc::Rc;
use crate::{Chunk, OpCode};
use crate::disasm::disassemble_instruction;
use crate::op::DecodeError;

pub struct VM {
    chunk: Rc<Chunk>,
    ip: usize,
}

#[derive(Debug)]
pub enum VMError {
    Decode(DecodeError),
    InvalidOpCode(u8),
    Runtime,
}

impl VM {
    pub fn new(chunk: Rc<Chunk>) -> VM {
        VM {
            chunk,
            ip: 0,
        }
    }

    pub fn run(&mut self) -> Result<(), VMError> {
        loop {
            #[cfg(feature = "trace_execution")]
            {
                disassemble_instruction(&mut std::io::stdout(), &self.chunk, self.ip).unwrap();
            }

            let (op, next_ip) = self.chunk.decode(self.ip).map_err(VMError::Decode)?;

            match op {
                OpCode::Constant(index) => {
                    // TODO return error
                    let value = self.chunk.constant(index).unwrap();

                    println!("{}", value);
                },
                OpCode::Return => return Ok(()),

                // TODO return error
                OpCode::Unknown(val) => return Err(VMError::InvalidOpCode(val)),
            }

            self.ip = next_ip
        }
    }
}