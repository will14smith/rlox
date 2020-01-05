use rlox_scanner::Token;
use rlox_parser::Expr;
use crate::Value;

pub fn evaluate(expr: &Expr) -> Value {
    match expr {
        Expr::Nil => Value::Nil,
        Expr::Boolean(value) => Value::Boolean(*value),
        Expr::Number(value) => Value::Number(*value),
        Expr::String(value) => Value::String(value.clone()),

        Expr::Grouping(expr) => evaluate(expr),

        Expr::Unary(op, expr) => {
            let value = evaluate(expr);

            match &op.token {
                Token::Minus => Value::Number(-value.as_number()),
                Token::Bang => Value::Boolean(!value.is_truthy()),

                _ => panic!("Invalid unary operation {:?}", op.token)
            }
        },

        Expr::Binary(_, _, _) => unimplemented!(),
    }
}

#[cfg(test)]
mod tests {
    use rlox_scanner::{ SourceToken };
    use super::*;

    fn tok_to_src(t: Token) -> SourceToken {
        SourceToken {
            token: t,
            lexeme: String::new(),
            line: 0
        }
    }

    #[test]
    fn test_literal() {
        assert_eq!(evaluate(&Expr::Nil), Value::Nil);
        assert_eq!(evaluate(&Expr::Boolean(true)), Value::Boolean(true));
        assert_eq!(evaluate(&Expr::Number(123f64)), Value::Number(123f64));
        assert_eq!(evaluate(&Expr::String("abc".into())), Value::String("abc".into()));
    }

    #[test]
    fn test_grouping() {
        assert_eq!(evaluate(&Expr::Grouping(Box::new(Expr::Boolean(true)))), Value::Boolean(true));
    }

    #[test]
    fn test_unary() {
        assert_eq!(evaluate(&Expr::Unary(tok_to_src(Token::Minus), Box::new(Expr::Number(123f64)))), Value::Number(-123f64));
        assert_eq!(evaluate(&Expr::Unary(tok_to_src(Token::Bang), Box::new(Expr::Boolean(true)))), Value::Boolean(false));
    }

    #[test]
    fn test_unary_runtime_error() {
        // TODO
        // evaluate(&Expr::Unary(tok_to_src(Token::Minus), Box::new(Expr::Boolean(true))))
        // evaluate(&Expr::Unary(tok_to_src(Token::Bang), Box::new(Expr::Number(123f64))))
    }
}