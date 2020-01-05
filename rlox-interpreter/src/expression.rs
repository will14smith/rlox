use rlox_parser::Expr;
use crate::Value;

pub fn evaluate(expr: &Expr) -> Value {
    match expr {
        Expr::Nil => Value::Nil,
        Expr::Boolean(value) => Value::Boolean(*value),
        Expr::Number(value) => Value::Number(*value),
        Expr::String(value) => Value::String(value.clone()),

        Expr::Binary(_, _, _) => unimplemented!(),
        Expr::Unary(_, _) => unimplemented!(),
        Expr::Grouping(_) => unimplemented!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_literal() {
        assert_eq!(evaluate(&Expr::Nil), Value::Nil);
        assert_eq!(evaluate(&Expr::Boolean(true)), Value::Boolean(true));
        assert_eq!(evaluate(&Expr::Number(123f64)), Value::Number(123f64));
        assert_eq!(evaluate(&Expr::String("abc".into())), Value::String("abc".into()));
    }
}