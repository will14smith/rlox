use rlox_scanner::{ SourceToken, Token };
use rlox_parser::Expr;
use crate::{EvaluateResult, Value, RuntimeError, RuntimeErrorDescription};

pub fn evaluate(expr: &Expr) -> EvaluateResult<Value> {
    match expr {
        Expr::Nil => Ok(Value::Nil),
        Expr::Boolean(value) => Ok(Value::Boolean(*value)),
        Expr::Number(value) => Ok(Value::Number(*value)),
        Expr::String(value) => Ok(Value::String(value.clone())),

        Expr::Grouping(expr) => evaluate(expr),

        Expr::Unary(op, expr) => {
            let value = evaluate(expr)?;

            match &op.token {
                Token::Minus => {
                    Ok(Value::Number(-cast_to_number(op, value)?))
                },
                Token::Bang => Ok(Value::Boolean(!value.is_truthy())),

                _ => panic!("Invalid unary operation {:?}", op.token)
            }
        },

        Expr::Binary(left_expr, op, right_expr) => {
            let left = evaluate(left_expr)?;
            let right = evaluate(right_expr)?;

            match &op.token {
                Token::Plus => match (left, right) {
                    (Value::Number(left), Value::Number(right)) => Ok(Value::Number(left + right)),
                    (Value::String(left), Value::String(right)) => Ok(Value::String(left + &right)),

                    (left, right) => Err(RuntimeError::new(op.clone(), RuntimeErrorDescription::InvalidAdditionArguments(left, right)))
                },
                Token::Minus => Ok(Value::Number(cast_to_number(op, left)? - cast_to_number(op, right)?)),
                Token::Star => Ok(Value::Number(cast_to_number(op, left)? * cast_to_number(op, right)?)),
                Token::Slash => Ok(Value::Number(cast_to_number(op, left)? / cast_to_number(op, right)?)),

                Token::Greater => Ok(Value::Boolean(cast_to_number(op, left)? > cast_to_number(op, right)?)),
                Token::GreaterEqual => Ok(Value::Boolean(cast_to_number(op, left)? >= cast_to_number(op, right)?)),
                Token::Less => Ok(Value::Boolean(cast_to_number(op, left)? < cast_to_number(op, right)?)),
                Token::LessEqual => Ok(Value::Boolean(cast_to_number(op, left)? <= cast_to_number(op, right)?)),

                Token::BangEqual => Ok(Value::Boolean(!left.is_equal(&right))),
                Token::EqualEqual => Ok(Value::Boolean(left.is_equal(&right))),

                _ => panic!("Invalid binary operation {:?}", op.token)
            }
        },
    }
}

fn cast_to_number(token: &SourceToken, value: Value) -> Result<f64, RuntimeError> {
    value.as_number().map_err(|_| RuntimeError::new(token.clone(), RuntimeErrorDescription::ExpectedNumber))
}

#[cfg(test)]
mod tests {
    use rlox_scanner::{ SourceToken };
    use super::*;

    fn evaluate_expect(expr: &Expr) -> Value {
        evaluate(expr).expect("Failed to evaluate expression")
    }

    fn tok_to_src(t: Token) -> SourceToken {
        SourceToken {
            token: t,
            lexeme: String::new(),
            line: 0
        }
    }

    #[test]
    fn test_literal() {
        assert_eq!(evaluate_expect(&Expr::Nil), Value::Nil);
        assert_eq!(evaluate_expect(&Expr::Boolean(true)), Value::Boolean(true));
        assert_eq!(evaluate_expect(&Expr::Number(123f64)), Value::Number(123f64));
        assert_eq!(evaluate_expect(&Expr::String("abc".into())), Value::String("abc".into()));
    }

    #[test]
    fn test_grouping() {
        assert_eq!(evaluate_expect(&Expr::Grouping(Box::new(Expr::Boolean(true)))), Value::Boolean(true));
    }

    #[test]
    fn test_unary() {
        assert_eq!(evaluate_expect(&Expr::Unary(tok_to_src(Token::Minus), Box::new(Expr::Number(123f64)))), Value::Number(-123f64));
        assert_eq!(evaluate_expect(&Expr::Unary(tok_to_src(Token::Bang), Box::new(Expr::Boolean(true)))), Value::Boolean(false));
    }

    #[test]
    fn test_unary_runtime_error() {
        let result = evaluate(&Expr::Unary(tok_to_src(Token::Minus), Box::new(Expr::Boolean(true))));
        assert!(result.is_err());
    }

    #[test]
    fn test_binary() {
        assert_eq!(evaluate_expect(&Expr::Binary(Box::new(Expr::Number(8f64)), tok_to_src(Token::Plus), Box::new(Expr::Number(4f64)))), Value::Number(12f64));
        assert_eq!(evaluate_expect(&Expr::Binary(Box::new(Expr::Number(8f64)), tok_to_src(Token::Minus), Box::new(Expr::Number(4f64)))), Value::Number(4f64));
        assert_eq!(evaluate_expect(&Expr::Binary(Box::new(Expr::Number(8f64)), tok_to_src(Token::Star), Box::new(Expr::Number(4f64)))), Value::Number(32f64));
        assert_eq!(evaluate_expect(&Expr::Binary(Box::new(Expr::Number(8f64)), tok_to_src(Token::Slash), Box::new(Expr::Number(4f64)))), Value::Number(2f64));

        assert_eq!(evaluate_expect(&Expr::Binary(Box::new(Expr::String("ab".into())), tok_to_src(Token::Plus), Box::new(Expr::String("cd".into())))), Value::String("abcd".into()));

        fn test_comparison(token: Token, lt: bool, eq: bool, gt: bool) {
            assert_eq!(evaluate_expect(&Expr::Binary(Box::new(Expr::Number(4f64)), tok_to_src(token.clone()), Box::new(Expr::Number(8f64)))), Value::Boolean(lt));
            assert_eq!(evaluate_expect(&Expr::Binary(Box::new(Expr::Number(4f64)), tok_to_src(token.clone()), Box::new(Expr::Number(4f64)))), Value::Boolean(eq));
            assert_eq!(evaluate_expect(&Expr::Binary(Box::new(Expr::Number(8f64)), tok_to_src(token.clone()), Box::new(Expr::Number(4f64)))), Value::Boolean(gt));
        }

        test_comparison(Token::Greater, false, false, true);
        test_comparison(Token::GreaterEqual, false, true, true);
        test_comparison(Token::Less, true, false, false);
        test_comparison(Token::LessEqual, true, true, false);

        assert_eq!(evaluate_expect(&Expr::Binary(Box::new(Expr::Number(4f64)), tok_to_src(Token::EqualEqual), Box::new(Expr::Number(4f64)))), Value::Boolean(true));
        assert_eq!(evaluate_expect(&Expr::Binary(Box::new(Expr::Number(4f64)), tok_to_src(Token::EqualEqual), Box::new(Expr::Number(8f64)))), Value::Boolean(false));
        assert_eq!(evaluate_expect(&Expr::Binary(Box::new(Expr::String("ab".into())), tok_to_src(Token::EqualEqual), Box::new(Expr::String("ab".into())))), Value::Boolean(true));
        assert_eq!(evaluate_expect(&Expr::Binary(Box::new(Expr::String("ab".into())), tok_to_src(Token::EqualEqual), Box::new(Expr::String("cd".into())))), Value::Boolean(false));
        assert_eq!(evaluate_expect(&Expr::Binary(Box::new(Expr::Boolean(true)), tok_to_src(Token::EqualEqual), Box::new(Expr::Boolean(true)))), Value::Boolean(true));
        assert_eq!(evaluate_expect(&Expr::Binary(Box::new(Expr::Boolean(true)), tok_to_src(Token::EqualEqual), Box::new(Expr::Boolean(false)))), Value::Boolean(false));
        assert_eq!(evaluate_expect(&Expr::Binary(Box::new(Expr::String("ab".into())), tok_to_src(Token::EqualEqual), Box::new(Expr::Number(4f64)))), Value::Boolean(false));
    }

    #[test]
    fn test_binary_runtime_error() {
        let result = evaluate(&Expr::Binary(Box::new(Expr::Number(8f64)), tok_to_src(Token::Minus), Box::new(Expr::String("cd".into()))));
        assert!(result.is_err());

        let result = evaluate(&Expr::Binary(Box::new(Expr::Number(8f64)), tok_to_src(Token::Greater), Box::new(Expr::String("cd".into()))));
        assert!(result.is_err());
    }
}