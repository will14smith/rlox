use std::{
    cell::RefCell,
    collections::HashMap,
    rc::Rc,
};
use rlox_scanner::{ SourceToken, Token };
use rlox_parser::Stmt;
use crate::{
    EvaluateResult,
    Value,
    RuntimeError,
    RuntimeErrorDescription
};
use crate::expression::evaluate;

pub struct Interpreter {
    environment: Rc<RefCell<Environment>>,
}

impl Interpreter {
    pub fn new() -> Interpreter {
        Interpreter {
            environment: Rc::new(RefCell::new(Environment::new())),
        }
    }

    pub fn interpret(&mut self, statements: Vec<Stmt>) -> EvaluateResult<()> {
        for statement in statements {
            self.evaluate_stmt(&statement)?;
        }

        Ok(())
    }

    fn evaluate_stmt(&mut self, stmt: &Stmt) -> EvaluateResult<()> {
        match stmt {
            Stmt::Expression(expr) => {
                evaluate(&mut self.environment.borrow_mut(), expr)?;

                Ok(())
            },
            Stmt::If(cond, then_branch, else_branch_opt) => {
                let cond_value = evaluate(&mut self.environment.borrow_mut(), cond)?;

                if cond_value.is_truthy() {
                    self.evaluate_stmt(then_branch)
                } else if let Some(else_branch) = else_branch_opt {
                    self.evaluate_stmt(else_branch)
                } else {
                    Ok(())
                }
            }
            Stmt::Print(expr) => {
                let value = evaluate(&mut self.environment.borrow_mut(), expr)?;
                println!("{}", value);

                Ok(())
            },
            Stmt::Var(name, initializer) => {
                let value = match initializer {
                    Some(expr) => evaluate(&mut self.environment.borrow_mut(), expr)?,
                    None => Value::Nil,
                };

                self.environment.borrow_mut().define(name.lexeme.clone(), value);

                Ok(())
            },
            Stmt::While(condition, body) => {
                while evaluate(&mut self.environment.borrow_mut(), condition)?.is_truthy() {
                    self.evaluate_stmt(body)?;
                }

                Ok(())
            },
            Stmt::Block(statements) => {
                let environment= Rc::new(RefCell::new(Environment::new_with_parent(Rc::clone(&self.environment))));

                self.evaluate_block(statements, environment)
            }
        }
    }

    fn evaluate_block(&mut self, statements: &Vec<Stmt>, mut environment: Rc<RefCell<Environment>>) -> EvaluateResult<()> {
        ::std::mem::swap(&mut self.environment, &mut environment);

        for statement in statements {
            match self.evaluate_stmt(statement) {
                Ok(()) => { }
                Err(err) => {
                    ::std::mem::swap(&mut self.environment, &mut environment);
                    return Err(err);
                }
            }
        }

        ::std::mem::swap(&mut self.environment, &mut environment);
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

    pub fn new_with_parent(parent: Rc<RefCell<Environment>>) -> Environment {
        Environment {
            parent: Some(parent),
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