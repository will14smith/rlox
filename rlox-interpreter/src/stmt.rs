use rlox_parser::Stmt;
use crate::{
    EvaluateResult,
    expression::evaluate as evaluate_expression
};

pub fn evaluate(stmt: &Stmt) -> EvaluateResult<()> {
    match stmt {
        Stmt::Expression(expr) => {
            evaluate_expression(expr)?;

            Ok(())
        },
        Stmt::Print(expr) => {
            let value = evaluate_expression(expr)?;
            println!("{}", value);

            Ok(())
        },
    }
}