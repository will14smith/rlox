use std::collections::HashMap;
use std::rc::Rc;
use crate::{Chunk, Object, OpCode, Value};
use crate::disasm::disassemble_instruction;
use crate::op::DecodeError;

pub struct VM {
    chunk: Rc<Chunk>,
    ip: usize,

    stack: Vec<Rc<Value>>,
    globals: HashMap<String, Rc<Value>>,
}

#[derive(Debug)]
pub enum VMError {
    Decode(DecodeError),
    InvalidOpCode(u8),
    InvalidConstant(u8, String),
    StackTooSmall(usize, usize),
    Runtime(usize, RuntimeError),
}

#[derive(Debug)]
pub enum RuntimeError {
    ExpectedNumber,
    ExpectedString,
    ExpectedIdentifier,
    UndefinedGlobal(String),
    UndefinedLocal(u8),
    InvalidAdditionArguments,
}

macro_rules! run_number_op {
    // entry cases
    ( $target:ident, $op:expr ; $($idents:ident),+ ) => {
        run_number_op!($target, $op, Value::Number; $($idents),+ ; 0);
    };
    ( $target:ident, $op:expr, $result:path ; $($idents:ident),+ ) => {
        run_number_op!($target, $op, $result; $($idents),+ ; 0);
    };

    // base case
    ( $target:ident, $op:expr, $result:path ; ; $count:expr ) => {
        {
            $target.drop($count)?;
            $target.push(Rc::new($result($op)));
        }
    };

    // final case (could this be removed?)
    ( $target:ident, $op:expr, $result:path ; $ident:ident ; $count:expr ) => {
        {
            let $ident = $target.as_number($target.peek($count)?.as_ref())?;
            run_number_op!($target, $op, $result ; ; $count + 1);
        }
    };

    // recursive case
    ( $target:ident, $op:expr, $result:path ; $ident:ident, $($idents:ident),* ; $count:expr ) => {
        {
            let $ident = $target.as_number($target.peek($count)?.as_ref())?;
            run_number_op!($target, $op, $result ; $($idents),* ; $count + 1);
        }
    };
}

impl VM {
    pub fn new(chunk: Rc<Chunk>) -> VM {
        VM {
            chunk,
            ip: 0,

            stack: Vec::new(),
            globals: HashMap::new(),
        }
    }

    pub fn run(&mut self) -> Result<(), VMError> {
        loop {
            #[cfg(feature = "trace_execution")]
            {
                self.print_stack();
                disassemble_instruction(&mut std::io::stderr(), &self.chunk, self.ip).unwrap();
            }

            let (op, mut next_ip) = self.chunk.decode(self.ip).map_err(VMError::Decode)?;

            match op {
                OpCode::Constant(index) => {
                    let value = self.chunk.constant(index).map_err(|e| VMError::InvalidConstant(index, e))?;
                    self.push(value);
                },
                OpCode::True => self.push(Rc::new(Value::Boolean(true))),
                OpCode::False => self.push(Rc::new(Value::Boolean(false))),
                OpCode::Nil => self.push(Rc::new(Value::Nil)),
                OpCode::Pop => { self.pop()?; },

                OpCode::GetLocal(index) => {
                    let value = self.stack.get(index as usize).map(Rc::clone);

                    match value {
                        Some(value) => self.push(value),
                        None => return Err(VMError::Runtime(self.chunk.line(self.ip), RuntimeError::UndefinedLocal(index))),
                    }
                },
                OpCode::SetLocal(index) => {
                    let value = self.peek(0)?;

                    std::mem::replace(&mut self.stack[index as usize], value);
                },
                OpCode::GetGlobal(index) => {
                    let ident = self.as_identifier(self.chunk.constant(index).map_err(|e| VMError::InvalidConstant(index, e))?.as_ref())?;
                    let value = self.globals.get(&ident).map(Rc::clone);

                    match value {
                        Some(value) => self.push(value),
                        None => return Err(VMError::Runtime(self.chunk.line(self.ip), RuntimeError::UndefinedGlobal(ident))),
                    }
                }
                OpCode::DefineGlobal(index) => {
                    let ident = self.as_identifier(self.chunk.constant(index).map_err(|e| VMError::InvalidConstant(index, e))?.as_ref())?;
                    let value = self.peek(0)?;

                    self.globals.insert(ident, value);
                    self.drop(1)?;
                }
                OpCode::SetGlobal(index) => {
                    let ident = self.as_identifier(self.chunk.constant(index).map_err(|e| VMError::InvalidConstant(index, e))?.as_ref())?;
                    let value = self.peek(0)?;

                    if !self.globals.contains_key(&ident) {
                        return Err(VMError::Runtime(self.chunk.line(self.ip), RuntimeError::UndefinedGlobal(ident)));
                    }

                    self.globals.insert(ident, value);
                }

                OpCode::Equal => {
                    let right = self.pop()?;
                    let left = self.pop()?;

                    let value = Value::Boolean(left.is_equal(right.as_ref()));

                    self.push(Rc::new(value));
                },
                OpCode::Greater => run_number_op!(self, left > right, Value::Boolean ; right, left),
                OpCode::Less => run_number_op!(self, left < right, Value::Boolean ; right, left),
                OpCode::Add => {
                    let right = self.peek(0)?;
                    let left = self.peek(1)?;

                    let result = if let (Value::Number(left), Value::Number(right)) = (left.as_ref(), right.as_ref()) {
                        Value::Number(left + right)
                    } else if let Ok(left) = self.as_string(left.as_ref()) {
                        Value::new_string(left.to_owned() + &right.to_string())
                    } else if let Ok(right) = self.as_string(right.as_ref()) {
                        Value::new_string(left.to_string() + &right)
                    } else {
                        return Err(VMError::Runtime(self.chunk.line(self.ip), RuntimeError::InvalidAdditionArguments))
                    };

                    self.drop(2)?;
                    self.push(Rc::new(result));
                },
                OpCode::Subtract => run_number_op!(self, left - right ; right, left),
                OpCode::Multiply => run_number_op!(self, left * right ; right, left),
                OpCode::Divide => run_number_op!(self, left / right ; right, left),
                OpCode::Not => {
                    let value = self.pop()?;
                    let new_value = Value::Boolean(!self.is_truthy(value.as_ref()));
                    self.push(Rc::new(new_value))
                },
                OpCode::Negate => run_number_op!(self, -value ; value),

                OpCode::Print => {
                    println!("{}", self.pop()?);
                },
                OpCode::Jump(offset) => {
                    next_ip = self.ip + offset as usize;
                },
                OpCode::JumpIfFalse(offset) => {
                    if !self.is_truthy(self.peek(0)?.as_ref()) {
                        next_ip = self.ip + offset as usize;
                    }
                },
                OpCode::Return => {
                    return Ok(())
                },

                // TODO return error
                OpCode::Unknown(val) => return Err(VMError::InvalidOpCode(val)),
            }

            self.ip = next_ip
        }
    }

    fn is_truthy(&self, value: &Value) -> bool {
        value.is_truthy()
    }

    fn as_number(&self, value: &Value) -> Result<f64, VMError> {
        value.as_number().map_err(|_| VMError::Runtime(self.chunk.line(self.ip), RuntimeError::ExpectedNumber))
    }
    fn as_string(&self, value: &Value) -> Result<String, VMError> {
        if let Value::Object(obj) = value {
            if let Object::String(s) = obj.as_ref() {
                return Ok(s.to_owned())
            }
        }

        Err(VMError::Runtime(self.chunk.line(self.ip), RuntimeError::ExpectedString))
    }
    fn as_identifier(&self, value: &Value) -> Result<String, VMError> {
        if let Value::Object(obj) = value {
            if let Object::String(s) = obj.as_ref() {
                return Ok(s.to_owned())
            }
        }

        Err(VMError::Runtime(self.chunk.line(self.ip), RuntimeError::ExpectedIdentifier))
    }
}

impl VM {
    fn push(&mut self, value: Rc<Value>) {
        self.stack.push(value)
    }

    fn peek(&self, offset: usize) -> Result<Rc<Value>, VMError> {
        let len = self.stack.len();

        if offset < len {
            Ok(Rc::clone(&self.stack[len - offset - 1]))
        } else {
            Err(VMError::StackTooSmall(offset + 1, self.stack.len()))
        }
    }

    fn pop(&mut self) -> Result<Rc<Value>, VMError> {
        self.stack.pop().ok_or(VMError::StackTooSmall(1, self.stack.len()))
    }

    fn drop(&mut self, count: usize) -> Result<(), VMError> {
        if count <= self.stack.len() {
            self.stack.drain(self.stack.len()-count..);

            Ok(())
        } else {
            Err(VMError::StackTooSmall(count, self.stack.len()))
        }
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