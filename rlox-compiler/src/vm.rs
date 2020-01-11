use std::rc::Rc;
use crate::{ Chunk, OpCode, Value };
use crate::disasm::disassemble_instruction;
use crate::op::DecodeError;

pub struct VM {
    chunk: Rc<Chunk>,
    ip: usize,

    stack: Vec<Rc<Value>>,
}

#[derive(Debug)]
pub enum VMError {
    Decode(DecodeError),
    InvalidOpCode(u8),
    InvalidConstant(u8, String),
    UnexpectedEmptyStack,
    Runtime(RuntimeError),
}

#[derive(Debug)]
pub enum RuntimeError {
    ExpectedNumber,
}

macro_rules! run_number_op {
    ( $target:ident, $op:expr ) => {
        $target.push(Rc::new(Value::Number($op)));
    };
    ( $target:ident, $ident:ident, $($rest:tt)+ ) => {
        {
            let $ident = as_number($target.pop()?.as_ref())?;
            run_number_op!($target, $($rest)+);
        }
    }
}

impl VM {
    pub fn new(chunk: Rc<Chunk>) -> VM {
        VM {
            chunk,
            ip: 0,

            stack: Vec::new(),
        }
    }

    pub fn run(&mut self) -> Result<(), VMError> {
        loop {
            #[cfg(feature = "trace_execution")]
            {
                self.print_stack();
                disassemble_instruction(&mut std::io::stderr(), &self.chunk, self.ip).unwrap();
            }

            let (op, next_ip) = self.chunk.decode(self.ip).map_err(VMError::Decode)?;

            match op {
                OpCode::Constant(index) => {
                    // TODO return error
                    let value = self.chunk.constant(index).map_err(|e| VMError::InvalidConstant(index, e))?;
                    self.push(value);
                },

                OpCode::Add => run_number_op!(self, right, left, left + right),
                OpCode::Subtract => run_number_op!(self, right, left, left - right),
                OpCode::Multiply => run_number_op!(self, right, left, left * right),
                OpCode::Divide => run_number_op!(self, right, left, left / right),
                OpCode::Negate => run_number_op!(self, value, -value),

                OpCode::Return => {
                    println!("{}", self.pop()?);
                    return Ok(())
                },

                // TODO return error
                OpCode::Unknown(val) => return Err(VMError::InvalidOpCode(val)),
            }

            self.ip = next_ip
        }
    }
}

impl VM {
    fn push(&mut self, value: Rc<Value>) {
        self.stack.push(value)
    }

    fn pop(&mut self) -> Result<Rc<Value>, VMError> {
        self.stack.pop().ok_or(VMError::UnexpectedEmptyStack)
    }

    #[cfg(feature = "trace_execution")]
    fn print_stack(&self) {
        eprint!("          ");
        for value in &self.stack {
            eprint!("[{}]", value);
        }
        eprintln!();
    }
}

fn as_number(value: &Value) -> Result<f64, VMError> {
    value.as_number().map_err(|_| VMError::Runtime(RuntimeError::ExpectedNumber))
}