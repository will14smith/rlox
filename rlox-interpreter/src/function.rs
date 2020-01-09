use std::{
    cell::RefCell,
    fmt::{ Display, Formatter, Error },
    rc::Rc,
};
use rlox_scanner::SourceToken;
use rlox_parser::{ Func, Stmt };
use crate::{RuntimeError, value::{Callable, Value}, Interpreter};
use crate::interpreter::Environment;

#[derive(Debug)]
pub struct FunctionDefinition {
    pub name: SourceToken,
    pub parameters: Vec<SourceToken>,
    pub body: Rc<Stmt>,
}

impl From<&Func> for FunctionDefinition {
    fn from(func: &Func) -> Self {
        FunctionDefinition {
            name: func.name.clone(),
            parameters: func.parameters.clone(),
            body: func.body.clone(),
        }
    }
}

impl Callable for FunctionDefinition {
    fn arity(&self) -> usize {
        self.parameters.len()
    }

    fn call(&self, interpreter: &mut Interpreter, arguments: Vec<Value>) -> Result<Value, RuntimeError> {
        let mut environment = Environment::new_with_parent(interpreter.global_environment());

        for (i, argument) in arguments.iter().enumerate() {
            let parameter = &self.parameters[i];
            environment.define(parameter.lexeme.clone(), argument.clone());
        }

        let environment = Rc::new(RefCell::new(environment));

        match &*self.body {
            Stmt::Block(stmts) => interpreter.evaluate_block(&stmts.iter().collect(), environment)?,
            stmt => interpreter.evaluate_block(&vec![stmt], environment)?
        }


        Ok(Value::Nil)
    }
}

impl Display for FunctionDefinition {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "<fn {}>", &self.name.lexeme)
    }
}