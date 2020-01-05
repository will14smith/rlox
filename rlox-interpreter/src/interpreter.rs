use rlox_parser::Stmt;
use crate::{
    EvaluateResult,
    expression::evaluate as evaluate_expression,
    stmt::evaluate as evaluate_stmt
};

pub struct Interpreter {
}

impl Interpreter {
    pub fn new() -> Interpreter {
        Interpreter {}
    }

    pub fn interpret(&mut self, statements: Vec<Stmt>) -> EvaluateResult<()> {
        for statement in statements {
            evaluate_stmt(&statement)?;
        }

        Ok(())
    }
}