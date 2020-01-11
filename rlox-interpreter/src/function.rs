use std::{
    cell::RefCell,
    fmt::{ Display, Formatter, Error },
    rc::Rc,
};
use rlox_scanner::SourceToken;
use rlox_parser::{ Func, Stmt };
use crate::{
    Interpreter,
    RuntimeError,

    interpreter::{Environment, StmtResult},
    value::{Callable, Value},
};

#[derive(Debug)]
pub struct FunctionDefinition {
    pub name: SourceToken,
    pub parameters: Vec<SourceToken>,
    pub body: Vec<Stmt>,
    pub closure: Rc<RefCell<Environment>>,
}

impl FunctionDefinition {
    pub fn new(func: &Func, closure: Rc<RefCell<Environment>>) -> FunctionDefinition {
        FunctionDefinition {
            name: func.name.clone(),
            parameters: func.parameters.clone(),
            body: func.body.clone(),
            closure,
        }

    }
}

impl Callable for FunctionDefinition {
    fn arity(&self) -> usize {
        self.parameters.len()
    }

    fn call(&self, interpreter: &mut Interpreter, arguments: Vec<Value>) -> Result<Value, RuntimeError> {
        let mut environment = Environment::new_with_parent(self.closure.clone());

        for (i, argument) in arguments.iter().enumerate() {
            let parameter = &self.parameters[i];
            environment.define(parameter.lexeme.clone(), argument.clone());
        }

        let environment = Rc::new(RefCell::new(environment));

        let result = interpreter.evaluate_block(&self.body, environment)?;
        let value = if let StmtResult::Return(value) = result { value } else { Value::Nil };

        Ok(value)
    }
}

impl Display for FunctionDefinition {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "<fn {}>", &self.name.lexeme)
    }
}