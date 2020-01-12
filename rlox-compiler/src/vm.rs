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
    StackTooSmall(usize, usize),
    Runtime(usize, RuntimeError),
}

#[derive(Debug)]
pub enum RuntimeError {
    ExpectedNumber,
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
                OpCode::True => self.push(Rc::new(Value::Boolean(true))),
                OpCode::False => self.push(Rc::new(Value::Boolean(false))),
                OpCode::Nil => self.push(Rc::new(Value::Nil)),

                OpCode::Equal => {
                    let right = self.pop()?;
                    let left = self.pop()?;

                    let value = Value::Boolean(left.is_equal(right.as_ref()));

                    self.push(Rc::new(value));
                },
                OpCode::Greater => run_number_op!(self, left > right, Value::Boolean ; right, left),
                OpCode::Less => run_number_op!(self, left < right, Value::Boolean ; right, left),
                OpCode::Add => run_number_op!(self, left + right ; right, left),
                OpCode::Subtract => run_number_op!(self, left - right ; right, left),
                OpCode::Multiply => run_number_op!(self, left * right ; right, left),
                OpCode::Divide => run_number_op!(self, left / right ; right, left),
                OpCode::Not => {
                    let value = self.pop()?;
                    let new_value = Value::Boolean(!self.is_truthy(value.as_ref()));
                    self.push(Rc::new(new_value))
                },
                OpCode::Negate => run_number_op!(self, -value ; value),

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

    fn is_truthy(&self, value: &Value) -> bool {
        value.is_truthy()
    }

    fn as_number(&self, value: &Value) -> Result<f64, VMError> {
        value.as_number().map_err(|_| VMError::Runtime(self.chunk.line(self.ip), RuntimeError::ExpectedNumber))
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