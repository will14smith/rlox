use rlox_scanner::{ SourceToken, Token };
use rlox_parser::Expr;
use crate::{
    interpreter::Environment,
    EvaluateResult,
    RuntimeError,
    RuntimeErrorDescription,
    Value,
};

pub fn evaluate(environment: &mut Environment, expr: &Expr) -> EvaluateResult<Value> {
    match expr {
        Expr::Nil => Ok(Value::Nil),
        Expr::Boolean(value) => Ok(Value::Boolean(*value)),
        Expr::Number(value) => Ok(Value::Number(*value)),
        Expr::String(value) => Ok(Value::String(value.clone())),

        Expr::Var(name) => {
            let value = environment.get(name)?;

            Ok(value.clone())
        },

        Expr::Grouping(expr) => evaluate(environment, expr),

        Expr::Unary(op, expr) => {
            let value = evaluate(environment, expr)?;

            match &op.token {
                Token::Minus => {
                    Ok(Value::Number(-cast_to_number(op, value)?))
                },
                Token::Bang => Ok(Value::Boolean(!value.is_truthy())),

                _ => panic!("Invalid unary operation {:?}", op.token)
            }
        },

        Expr::Binary(left_expr, op, right_expr) => {
            let left = evaluate(environment, left_expr)?;
            let right = evaluate(environment, right_expr)?;

            match &op.token {
                Token::Plus => match (left, right) {
                    (Value::Number(left), Value::Number(right)) => Ok(Value::Number(left + right)),
                    (Value::String(left), right) => Ok(Value::String(left + &right.to_string())),
                    (left, Value::String(right)) => Ok(Value::String(left.to_string() + &right)),

                    (left, right) => Err(RuntimeError::new(op.clone(), RuntimeErrorDescription::InvalidAdditionArguments(left, right)))
                },
                Token::Minus => Ok(Value::Number(cast_to_number(op, left)? - cast_to_number(op, right)?)),
                Token::Star => Ok(Value::Number(cast_to_number(op, left)? * cast_to_number(op, right)?)),
                Token::Slash => {
                    let left = cast_to_number(op, left)?;
                    let right = cast_to_number(op, right)?;

                    if right == 0f64 {
                        Err(RuntimeError::new(op.clone(), RuntimeErrorDescription::DivideByZero))
                    } else {
                        Ok(Value::Number(left / right))
                    }
                },

                Token::Greater => Ok(Value::Boolean(cast_to_number(op, left)? > cast_to_number(op, right)?)),
                Token::GreaterEqual => Ok(Value::Boolean(cast_to_number(op, left)? >= cast_to_number(op, right)?)),
                Token::Less => Ok(Value::Boolean(cast_to_number(op, left)? < cast_to_number(op, right)?)),
                Token::LessEqual => Ok(Value::Boolean(cast_to_number(op, left)? <= cast_to_number(op, right)?)),

                Token::BangEqual => Ok(Value::Boolean(!left.is_equal(&right))),
                Token::EqualEqual => Ok(Value::Boolean(left.is_equal(&right))),

                _ => panic!("Invalid binary operation {:?}", op.token)
            }
        },

        Expr::Assign(name, expr) => {
            let value = evaluate(environment, expr)?;

            environment.assign(name, value.clone())?;

            Ok(value)
        }
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
        let mut environment = Environment::new();
        evaluate(&mut environment, expr).expect("Failed to evaluate expression")
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
        let mut environment = Environment::new();

        let result = evaluate(&mut environment, &Expr::Unary(tok_to_src(Token::Minus), Box::new(Expr::Boolean(true))));
        assert!(result.is_err());
    }

    #[test]
    fn test_binary() {
        assert_eq!(evaluate_expect(&Expr::Binary(Box::new(Expr::Number(8f64)), tok_to_src(Token::Plus), Box::new(Expr::Number(4f64)))), Value::Number(12f64));
        assert_eq!(evaluate_expect(&Expr::Binary(Box::new(Expr::Number(8f64)), tok_to_src(Token::Minus), Box::new(Expr::Number(4f64)))), Value::Number(4f64));
        assert_eq!(evaluate_expect(&Expr::Binary(Box::new(Expr::Number(8f64)), tok_to_src(Token::Star), Box::new(Expr::Number(4f64)))), Value::Number(32f64));
        assert_eq!(evaluate_expect(&Expr::Binary(Box::new(Expr::Number(8f64)), tok_to_src(Token::Slash), Box::new(Expr::Number(4f64)))), Value::Number(2f64));

        assert_eq!(evaluate_expect(&Expr::Binary(Box::new(Expr::String("ab".into())), tok_to_src(Token::Plus), Box::new(Expr::String("cd".into())))), Value::String("abcd".into()));
        assert_eq!(evaluate_expect(&Expr::Binary(Box::new(Expr::String("ab".into())), tok_to_src(Token::Plus), Box::new(Expr::Number(34f64)))), Value::String("ab34".into()));
        assert_eq!(evaluate_expect(&Expr::Binary(Box::new(Expr::Number(12f64)), tok_to_src(Token::Plus), Box::new(Expr::String("cd".into())))), Value::String("12cd".into()));

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
        let mut environment = Environment::new();

        let result = evaluate(&mut environment, &Expr::Binary(Box::new(Expr::Number(8f64)), tok_to_src(Token::Minus), Box::new(Expr::String("cd".into()))));
        assert!(result.is_err());

        let result = evaluate(&mut environment, &Expr::Binary(Box::new(Expr::Number(8f64)), tok_to_src(Token::Greater), Box::new(Expr::String("cd".into()))));
        assert!(result.is_err());

        let result = evaluate(&mut environment, &Expr::Binary(Box::new(Expr::Number(8f64)), tok_to_src(Token::Slash), Box::new(Expr::Number(0f64))));
        assert!(result.is_err());
    }

    #[test]
    fn test_assign() {
        let mut environment = Environment::new();
        environment.define("a".into(), Value::Nil);
        
        let a = tok_to_src(Token::Identifier("a".into()));
        evaluate(&mut environment, &Expr::Assign(a.clone(), Box::new(Expr::Boolean(true)))).unwrap();

        assert_eq!(environment.get(&a), Ok(&Value::Boolean(true)));

        let result = evaluate(&mut environment, &Expr::Assign(tok_to_src(Token::Identifier("b".into())), Box::new(Expr::Boolean(true))));
        assert!(result.is_err());

    }
}