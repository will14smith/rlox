use std::collections::HashMap;
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
    values: HashMap<String, Value>,
}

impl Environment {
    pub fn new() -> Environment {
        Environment {
            values: HashMap::new(),
        }
    }

    pub fn get(&self, token: &SourceToken) -> EvaluateResult<&Value> {
        match self.values.get(Self::get_identifier_name(token)) {
            Some(value) => Ok(value),
            None => Err(RuntimeError::new(token.clone(), RuntimeErrorDescription::UndefinedVariable)),
        }
    }

    pub fn define(&mut self, name: String, value: Value) {
        self.values.insert(name, value);
    }

    pub fn assign(&mut self, token: &SourceToken, value: Value) -> EvaluateResult<()> {
        let name = Self::get_identifier_name(token);

        if self.values.contains_key(name) {
            self.values.insert(name.clone(), value);

            Ok(())
        } else {
            Err(RuntimeError::new(token.clone(), RuntimeErrorDescription::UndefinedVariable))
        }
    }

    fn get_identifier_name(token: &SourceToken) -> &String {
        match &token.token {
            Token::Identifier(value) => value,

            t => panic!("Invalid token {:?} for variable name", t),
        }
    }
}