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

        Expr::Binary(left_expr, op, right_expr) => {
            let left = evaluate(left_expr);
            let right = evaluate(right_expr);

            match &op.token {
                Token::Plus => match (left, right) {
                    (Value::Number(left), Value::Number(right)) => Value::Number(left + right),
                    (Value::String(left), Value::String(right)) => Value::String(left + &right),

                    (left, right) => panic!("Invalid operands for addition {:?} + {:?}", left, right)
                },
                Token::Minus => Value::Number(left.as_number() - right.as_number()),
                Token::Star => Value::Number(left.as_number() * right.as_number()),
                Token::Slash => Value::Number(left.as_number() / right.as_number()),

                Token::Greater => Value::Boolean(left.as_number() > right.as_number()),
                Token::GreaterEqual => Value::Boolean(left.as_number() >= right.as_number()),
                Token::Less => Value::Boolean(left.as_number() < right.as_number()),
                Token::LessEqual => Value::Boolean(left.as_number() <= right.as_number()),

                Token::BangEqual => Value::Boolean(!left.is_equal(&right)),
                Token::EqualEqual => Value::Boolean(left.is_equal(&right)),

                _ => panic!("Invalid binary operation {:?}", op.token)
            }
        },
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

    #[test]
    fn test_binary() {
        assert_eq!(evaluate(&Expr::Binary(Box::new(Expr::Number(8f64)), tok_to_src(Token::Plus), Box::new(Expr::Number(4f64)))), Value::Number(12f64));
        assert_eq!(evaluate(&Expr::Binary(Box::new(Expr::Number(8f64)), tok_to_src(Token::Minus), Box::new(Expr::Number(4f64)))), Value::Number(4f64));
        assert_eq!(evaluate(&Expr::Binary(Box::new(Expr::Number(8f64)), tok_to_src(Token::Star), Box::new(Expr::Number(4f64)))), Value::Number(32f64));
        assert_eq!(evaluate(&Expr::Binary(Box::new(Expr::Number(8f64)), tok_to_src(Token::Slash), Box::new(Expr::Number(4f64)))), Value::Number(2f64));

        assert_eq!(evaluate(&Expr::Binary(Box::new(Expr::String("ab".into())), tok_to_src(Token::Plus), Box::new(Expr::String("cd".into())))), Value::String("abcd".into()));

        fn test_comparison(token: Token, lt: bool, eq: bool, gt: bool) {
            assert_eq!(evaluate(&Expr::Binary(Box::new(Expr::Number(4f64)), tok_to_src(token.clone()), Box::new(Expr::Number(8f64)))), Value::Boolean(lt));
            assert_eq!(evaluate(&Expr::Binary(Box::new(Expr::Number(4f64)), tok_to_src(token.clone()), Box::new(Expr::Number(4f64)))), Value::Boolean(eq));
            assert_eq!(evaluate(&Expr::Binary(Box::new(Expr::Number(8f64)), tok_to_src(token.clone()), Box::new(Expr::Number(4f64)))), Value::Boolean(gt));
        }

        test_comparison(Token::Greater, false, false, true);
        test_comparison(Token::GreaterEqual, false, true, true);
        test_comparison(Token::Less, true, false, false);
        test_comparison(Token::LessEqual, true, true, false);

        assert_eq!(evaluate(&Expr::Binary(Box::new(Expr::Number(4f64)), tok_to_src(Token::EqualEqual), Box::new(Expr::Number(4f64)))), Value::Boolean(true));
        assert_eq!(evaluate(&Expr::Binary(Box::new(Expr::Number(4f64)), tok_to_src(Token::EqualEqual), Box::new(Expr::Number(8f64)))), Value::Boolean(false));
        assert_eq!(evaluate(&Expr::Binary(Box::new(Expr::String("ab".into())), tok_to_src(Token::EqualEqual), Box::new(Expr::String("ab".into())))), Value::Boolean(true));
        assert_eq!(evaluate(&Expr::Binary(Box::new(Expr::String("ab".into())), tok_to_src(Token::EqualEqual), Box::new(Expr::String("cd".into())))), Value::Boolean(false));
        assert_eq!(evaluate(&Expr::Binary(Box::new(Expr::Boolean(true)), tok_to_src(Token::EqualEqual), Box::new(Expr::Boolean(true)))), Value::Boolean(true));
        assert_eq!(evaluate(&Expr::Binary(Box::new(Expr::Boolean(true)), tok_to_src(Token::EqualEqual), Box::new(Expr::Boolean(false)))), Value::Boolean(false));
        assert_eq!(evaluate(&Expr::Binary(Box::new(Expr::String("ab".into())), tok_to_src(Token::EqualEqual), Box::new(Expr::Number(4f64)))), Value::Boolean(false));
    }

    #[test]
    fn test_binary_runtime_error() {
        // TODO
        // evaluate(&Expr::Binary(Box::new(Expr::Number(8f64)), tok_to_src(Token::Minus), Box::new(Expr::String("cd".into()))))
        // evaluate(&Expr::Binary(Box::new(Expr::Number(8f64)), tok_to_src(Token::Greater), Box::new(Expr::String("cd".into()))))
    }
}