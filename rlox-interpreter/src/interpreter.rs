use std::collections::HashMap;
use rlox_scanner::SourceToken;
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
        match self.values.get(&token.lexeme) {
            Some(value) => Ok(value),
            None => Err(RuntimeError::new(token.clone(), RuntimeErrorDescription::UndefinedVariable)),
        }
    }

    pub fn define(&mut self, name: String, value: Value) {
        self.values.insert(name, value);
    }
}