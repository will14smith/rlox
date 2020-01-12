use rlox_parser::Expr;
use crate::{Chunk, Value, OpCode};
use rlox_scanner::Token;

pub struct Compiler<'a> {
    chunk: &'a mut Chunk,
}

#[derive(Debug)]
pub enum CompilerError {
    TooManyConstants,
}

impl<'a> Compiler<'a> {
    pub fn new(chunk: &'a mut Chunk) -> Compiler<'a> {
        Compiler {
            chunk,
        }
    }
}

impl<'a> Compiler<'a> {
    pub fn compile(&mut self, expr: Expr) -> Result<(), CompilerError> {
        self.compile_expr(expr)?;

        self.chunk.add(OpCode::Return, 0);

        Ok(())
    }

    fn compile_expr(&mut self, expr: Expr) -> Result<(), CompilerError> {
        match expr {
            Expr::Assign(_, _) => unimplemented!(),
            Expr::Binary(left, op, right) => {
                self.compile_expr(*left)?;
                self.compile_expr(*right)?;

                match &op.token {
                    Token::BangEqual => { self.chunk.add(OpCode::Equal, op.line); self.chunk.add(OpCode::Not, op.line) },
                    Token::EqualEqual => self.chunk.add(OpCode::Equal, op.line),
                    Token::Greater => self.chunk.add(OpCode::Greater, op.line),
                    Token::GreaterEqual => { self.chunk.add(OpCode::Less, op.line); self.chunk.add(OpCode::Not, op.line) },
                    Token::Less => self.chunk.add(OpCode::Less, op.line),
                    Token::LessEqual => { self.chunk.add(OpCode::Greater, op.line); self.chunk.add(OpCode::Not, op.line) },

                    Token::Plus => self.chunk.add(OpCode::Add, op.line),
                    Token::Minus => self.chunk.add(OpCode::Subtract, op.line),
                    Token::Star => self.chunk.add(OpCode::Multiply, op.line),
                    Token::Slash => self.chunk.add(OpCode::Divide, op.line),

                    _ => panic!("Invalid binary operation {:?}", op.token)
                }

                Ok(())
            },
            Expr::Call(_, _, _) => unimplemented!(),
            Expr::Logical(_, _, _) => unimplemented!(),
            Expr::Unary(op, value) => {
                self.compile_expr(*value)?;

                match &op.token {
                    Token::Bang => self.chunk.add(OpCode::Not, op.line),
                    Token::Minus => self.chunk.add(OpCode::Negate, op.line),

                    _ => panic!("Invalid unary operation {:?}", op.token)
                }

                Ok(())
            },
            Expr::Grouping(expr) => self.compile_expr(*expr),
            Expr::Var(_) => unimplemented!(),
            Expr::String(_, _) => unimplemented!(),
            Expr::Number(token, value) => {
                let constant = self.chunk.add_constant(Value::Number(value)).map_err(|_| CompilerError::TooManyConstants)?;
                self.chunk.add(OpCode::Constant(constant), token.line);
                Ok(())
            },
            Expr::Boolean(token, value) => {
                self.chunk.add(if value { OpCode::True } else { OpCode::False }, token.line);
                Ok(())
            },
            Expr::Nil(token) => {
                self.chunk.add(OpCode::Nil, token.line);
                Ok(())
            },
        }
    }
}