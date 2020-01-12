use rlox_scanner::{ SourceToken, Token };
use rlox_parser::Expr;
use crate::{
    EvaluateResult,
    Interpreter,
    RuntimeError,
    RuntimeErrorDescription,
    Value,
};

pub fn evaluate(interpreter: &mut Interpreter, expr: &Expr) -> EvaluateResult<Value> {
    match expr {
        Expr::Nil(_) => Ok(Value::Nil),
        Expr::Boolean(_, value) => Ok(Value::Boolean(*value)),
        Expr::Number(_, value) => Ok(Value::Number(*value)),
        Expr::String(_, value) => Ok(Value::String(value.clone())),

        Expr::Var(name) => {
            let value = interpreter.environment().borrow().get(name)?;

            Ok((*value).clone())
        },

        Expr::Grouping(expr) => evaluate(interpreter, expr),

        Expr::Unary(op, expr) => {
            let value = evaluate(interpreter, expr)?;

            match &op.token {
                Token::Minus => {
                    Ok(Value::Number(-cast_to_number(op, value)?))
                },
                Token::Bang => Ok(Value::Boolean(!value.is_truthy())),

                _ => panic!("Invalid unary operation {:?}", op.token)
            }
        },

        Expr::Logical(left_expr, op, right_expr) => {
            let left = evaluate(interpreter, left_expr)?;

            match &op.token {
                Token::Or => if left.is_truthy() { Ok(left) } else { evaluate(interpreter, right_expr) },
                Token::And => if !left.is_truthy() { Ok(left) } else { evaluate(interpreter, right_expr) },

                _ => panic!("Invalid logical operation {:?}", op.token)
            }
        },

        Expr::Binary(left_expr, op, right_expr) => {
            let left = evaluate(interpreter, left_expr)?;
            let right = evaluate(interpreter, right_expr)?;

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

        Expr::Call(callee_expr, paren, argument_exprs) => {
            let callee = evaluate(interpreter, callee_expr)?;

            let mut arguments = Vec::new();
            for expr in argument_exprs {
                let argument = evaluate(interpreter, expr)?;
                arguments.push(argument);
            }

            let function = callee.as_callable()
                .map_err(|_| RuntimeError::new(paren.clone(), RuntimeErrorDescription::CalleeNotCallable))?;

            if arguments.len() != function.arity() {
                return Err(RuntimeError::new(paren.clone(), RuntimeErrorDescription::UnexpectedNumberOfArguments { expected: function.arity(), provided: arguments.len() }))
            }

            function.call(interpreter, arguments)
        },

        Expr::Assign(name, expr) => {
            let value = evaluate(interpreter, expr)?;

            interpreter.environment().borrow_mut().assign(name, value.clone())?;

            Ok(value)
        }
    }
}

fn cast_to_number(token: &SourceToken, value: Value) -> Result<f64, RuntimeError> {
    value.as_number().map_err(|_| RuntimeError::new(token.clone(), RuntimeErrorDescription::ExpectedNumber))
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;
    use rlox_scanner::{ SourceToken };

    use super::*;

    fn evaluate_expect(expr: &Expr) -> Value {
        let mut interpreter = Interpreter::new();
        evaluate(&mut interpreter, expr).expect("Failed to evaluate expression")
    }

    fn tok_to_src(t: Token) -> SourceToken {
        SourceToken {
            token: t,
            lexeme: String::new(),
            line: 0
        }
    }

    fn expr_num(n: f64) -> Expr {
        Expr::Number(tok_to_src(Token::Number(n)), n)
    }
    fn expr_bool(b: bool) -> Expr {
        Expr::Boolean(tok_to_src(if b { Token::True } else { Token::False }), b)
    }
    fn expr_str(s: &str) -> Expr {
        Expr::String(tok_to_src(Token::String(s.into())), s.into())
    }
    
    #[test]
    fn test_literal() {
        assert_eq!(evaluate_expect(&Expr::Nil(tok_to_src(Token::Nil))), Value::Nil);
        assert_eq!(evaluate_expect(&expr_bool(true)), Value::Boolean(true));
        assert_eq!(evaluate_expect(&expr_num(123f64)), Value::Number(123f64));
        assert_eq!(evaluate_expect(&expr_str("abc".into())), Value::String("abc".into()));
    }

    #[test]
    fn test_grouping() {
        assert_eq!(evaluate_expect(&Expr::Grouping(Box::new(expr_bool(true)))), Value::Boolean(true));
    }

    #[test]
    fn test_unary() {
        assert_eq!(evaluate_expect(&Expr::Unary(tok_to_src(Token::Minus), Box::new(expr_num(123f64)))), Value::Number(-123f64));
        assert_eq!(evaluate_expect(&Expr::Unary(tok_to_src(Token::Bang), Box::new(expr_bool(true)))), Value::Boolean(false));
    }

    #[test]
    fn test_unary_runtime_error() {
        let mut interpreter = Interpreter::new();

        let result = evaluate(&mut interpreter, &Expr::Unary(tok_to_src(Token::Minus), Box::new(expr_bool(true))));
        assert!(result.is_err());
    }

    #[test]
    fn test_binary() {
        assert_eq!(evaluate_expect(&Expr::Binary(Box::new(expr_num(8f64)), tok_to_src(Token::Plus), Box::new(expr_num(4f64)))), Value::Number(12f64));
        assert_eq!(evaluate_expect(&Expr::Binary(Box::new(expr_num(8f64)), tok_to_src(Token::Minus), Box::new(expr_num(4f64)))), Value::Number(4f64));
        assert_eq!(evaluate_expect(&Expr::Binary(Box::new(expr_num(8f64)), tok_to_src(Token::Star), Box::new(expr_num(4f64)))), Value::Number(32f64));
        assert_eq!(evaluate_expect(&Expr::Binary(Box::new(expr_num(8f64)), tok_to_src(Token::Slash), Box::new(expr_num(4f64)))), Value::Number(2f64));

        assert_eq!(evaluate_expect(&Expr::Binary(Box::new(expr_str("ab".into())), tok_to_src(Token::Plus), Box::new(expr_str("cd".into())))), Value::String("abcd".into()));
        assert_eq!(evaluate_expect(&Expr::Binary(Box::new(expr_str("ab".into())), tok_to_src(Token::Plus), Box::new(expr_num(34f64)))), Value::String("ab34".into()));
        assert_eq!(evaluate_expect(&Expr::Binary(Box::new(expr_num(12f64)), tok_to_src(Token::Plus), Box::new(expr_str("cd".into())))), Value::String("12cd".into()));

        fn test_comparison(token: Token, lt: bool, eq: bool, gt: bool) {
            assert_eq!(evaluate_expect(&Expr::Binary(Box::new(expr_num(4f64)), tok_to_src(token.clone()), Box::new(expr_num(8f64)))), Value::Boolean(lt));
            assert_eq!(evaluate_expect(&Expr::Binary(Box::new(expr_num(4f64)), tok_to_src(token.clone()), Box::new(expr_num(4f64)))), Value::Boolean(eq));
            assert_eq!(evaluate_expect(&Expr::Binary(Box::new(expr_num(8f64)), tok_to_src(token.clone()), Box::new(expr_num(4f64)))), Value::Boolean(gt));
        }

        test_comparison(Token::Greater, false, false, true);
        test_comparison(Token::GreaterEqual, false, true, true);
        test_comparison(Token::Less, true, false, false);
        test_comparison(Token::LessEqual, true, true, false);

        assert_eq!(evaluate_expect(&Expr::Binary(Box::new(expr_num(4f64)), tok_to_src(Token::EqualEqual), Box::new(expr_num(4f64)))), Value::Boolean(true));
        assert_eq!(evaluate_expect(&Expr::Binary(Box::new(expr_num(4f64)), tok_to_src(Token::EqualEqual), Box::new(expr_num(8f64)))), Value::Boolean(false));
        assert_eq!(evaluate_expect(&Expr::Binary(Box::new(expr_str("ab".into())), tok_to_src(Token::EqualEqual), Box::new(expr_str("ab".into())))), Value::Boolean(true));
        assert_eq!(evaluate_expect(&Expr::Binary(Box::new(expr_str("ab".into())), tok_to_src(Token::EqualEqual), Box::new(expr_str("cd".into())))), Value::Boolean(false));
        assert_eq!(evaluate_expect(&Expr::Binary(Box::new(expr_bool(true)), tok_to_src(Token::EqualEqual), Box::new(expr_bool(true)))), Value::Boolean(true));
        assert_eq!(evaluate_expect(&Expr::Binary(Box::new(expr_bool(true)), tok_to_src(Token::EqualEqual), Box::new(expr_bool(false)))), Value::Boolean(false));
        assert_eq!(evaluate_expect(&Expr::Binary(Box::new(expr_str("ab".into())), tok_to_src(Token::EqualEqual), Box::new(expr_num(4f64)))), Value::Boolean(false));
    }

    #[test]
    fn test_binary_runtime_error() {
        let mut interpreter = Interpreter::new();

        let result = evaluate(&mut interpreter, &Expr::Binary(Box::new(expr_num(8f64)), tok_to_src(Token::Minus), Box::new(expr_str("cd".into()))));
        assert!(result.is_err());

        let result = evaluate(&mut interpreter, &Expr::Binary(Box::new(expr_num(8f64)), tok_to_src(Token::Greater), Box::new(expr_str("cd".into()))));
        assert!(result.is_err());

        let result = evaluate(&mut interpreter, &Expr::Binary(Box::new(expr_num(8f64)), tok_to_src(Token::Slash), Box::new(expr_num(0f64))));
        assert!(result.is_err());
    }

    #[test]
    fn test_assign() {
        let mut interpreter = Interpreter::new();
        interpreter.environment().borrow_mut().define("a".into(), Value::Nil);

        let a = tok_to_src(Token::Identifier("a".into()));
        evaluate(&mut interpreter, &Expr::Assign(a.clone(), Box::new(expr_bool(true)))).unwrap();

        assert_eq!(interpreter.environment().borrow().get(&a), Ok(Rc::new(Value::Boolean(true))));

        let result = evaluate(&mut interpreter, &Expr::Assign(tok_to_src(Token::Identifier("b".into())), Box::new(expr_bool(true))));
        assert!(result.is_err());

    }
}