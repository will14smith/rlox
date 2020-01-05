use rlox_parser::Stmt;
use crate::{
    interpreter::Environment,
    EvaluateResult,
    expression::evaluate as evaluate_expression,
    Value,
};

pub fn evaluate(environment: &mut Environment, stmt: &Stmt) -> EvaluateResult<()> {
    match stmt {
        Stmt::Expression(expr) => {
            evaluate_expression(environment, expr)?;

            Ok(())
        },
        Stmt::Print(expr) => {
            let value = evaluate_expression(environment, expr)?;
            println!("{}", value);

            Ok(())
        },
        Stmt::Var(name, initializer) => {
            let value = match initializer {
                Some(expr) => evaluate_expression(environment, expr)?,
                None => Value::Nil,
            };

            environment.define(name.lexeme.clone(), value);

            Ok(())
        }
    }
}