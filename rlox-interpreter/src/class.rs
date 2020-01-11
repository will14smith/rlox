use std::fmt::{Display, Formatter, Error};
use rlox_scanner::SourceToken;
use rlox_parser::Func;
use crate::{
    Interpreter,
    RuntimeError,

    value::{ Callable, Value },
};
use crate::function::FunctionDefinition;

#[derive(Debug)]
pub struct ClassDefinition {
    name: SourceToken
}

impl ClassDefinition {
    pub fn new(name: &SourceToken, functions: &Vec<Func>) -> ClassDefinition {
        unimplemented!()
    }
}

impl Callable for ClassDefinition {
    fn arity(&self) -> usize {
        unimplemented!()
    }

    fn call(&self, interpreter: &mut Interpreter, arguments: Vec<Value>) -> Result<Value, RuntimeError> {
        unimplemented!()
    }
}

impl Display for ClassDefinition {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", self.name.lexeme)
    }
}