use std::{
    cell::RefCell,
    collections::HashMap,
    rc::Rc,
};
use rlox_scanner::{ SourceToken, Token };
use rlox_parser::Stmt;
use crate::{
    EvaluateResult,
    stmt::evaluate as evaluate_stmt,
    Value,
    RuntimeError,
    RuntimeErrorDescription
};

pub struct Interpreter {
    environment: Environment,
}

impl Interpreter {
    pub fn new() -> Interpreter {
        Interpreter {
            environment: Environment::new(),
        }
    }

    pub fn interpret(&mut self, statements: Vec<Stmt>) -> EvaluateResult<()> {
        for statement in statements {
            evaluate_stmt(&mut self.environment, &statement)?;
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct Environment {
    parent: Option<Rc<RefCell<Environment>>>,
    values: HashMap<String, Rc<Value>>,
}

impl Environment {
    pub fn new() -> Environment {
        Environment {
            parent: None,
            values: HashMap::new(),
        }
    }

    pub fn get(&self, token: &SourceToken) -> EvaluateResult<Rc<Value>> {
        match self.values.get(Self::get_identifier_name(token)) {
            Some(value) => Ok(Rc::clone(value)),
            None => {
                match &self.parent {
                    Some(parent) => {
                        let env = parent.borrow();

                        env.get(token)
                    },
                    None => Err(RuntimeError::new(token.clone(), RuntimeErrorDescription::UndefinedVariable))
                }
            },
        }
    }

    pub fn define(&mut self, name: String, value: Value) {
        self.values.insert(name, Rc::new(value));
    }

    pub fn assign(&mut self, token: &SourceToken, value: Value) -> EvaluateResult<()> {
        let name = Self::get_identifier_name(token);

        if self.values.contains_key(name) {
            self.values.insert(name.clone(), Rc::new(value));

            Ok(())
        } else {
            match &self.parent {
                Some(parent) => {
                    let mut env = parent.borrow_mut();

                    env.assign(token, value)
                },
                None => Err(RuntimeError::new(token.clone(), RuntimeErrorDescription::UndefinedVariable))
            }
        }
    }

    fn get_identifier_name(token: &SourceToken) -> &String {
        match &token.token {
            Token::Identifier(value) => value,

            t => panic!("Invalid token {:?} for variable name", t),
        }
    }
}