use std::rc::Rc;
use rlox_scanner::Token;
use rlox_parser::{Expr, Stmt};
use crate::{Chunk, Object, OpCode, Value};

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
    pub fn compile(&mut self, statements: Vec<Stmt>) -> Result<(), CompilerError> {
        for statement in statements {
            self.compile_stmt(statement)?;
        }
        
        Ok(())
    }

    fn compile_stmt(&mut self, stmt: Stmt) -> Result<(), CompilerError> {
        match stmt {
            Stmt::Block(_) => unimplemented!(),
            Stmt::Class(_, _) => unimplemented!(),
            Stmt::Expression(expr) => {
                self.compile_expr(expr)?;
                self.chunk.add(OpCode::Pop, 0); // TODO get line
            },
            Stmt::Function(_) => unimplemented!(),
            Stmt::If(_, _, _) => unimplemented!(),
            Stmt::Print(expr) => {
                self.compile_expr(expr)?;
                self.chunk.add(OpCode::Print, 0); // TODO get line
            },
            Stmt::Return(_, _) => unimplemented!(),
            Stmt::Var(name, expr) => {
                if let Some(expr) = expr {
                    self.compile_expr(expr)?;
                } else {
                    self.chunk.add(OpCode::Nil, name.line);
                }

                let constant = self.add_string(name.lexeme)?;
                self.chunk.add(OpCode::DefineGlobal(constant), 0);
            },
            Stmt::While(_, _) => unimplemented!(),
        }

        Ok(())
    }
    
    fn compile_expr(&mut self, expr: Expr) -> Result<(), CompilerError> {
        match expr {
            Expr::Assign(name, value) => {
                self.compile_expr(*value)?;
                let constant = self.add_string(name.lexeme)?;
                self.chunk.add(OpCode::SetGlobal(constant), name.line);
            },
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
            },
            Expr::Grouping(expr) => self.compile_expr(*expr)?,
            Expr::Var(name) => {
                let constant = self.add_string(name.lexeme)?;
                self.chunk.add(OpCode::GetGlobal(constant), name.line);
            },
            Expr::String(token, value) => {
                let constant = self.add_string(value)?;
                self.chunk.add(OpCode::Constant(constant), token.line);
            },
            Expr::Number(token, value) => {
                let constant = self.chunk.add_constant(Value::Number(value)).map_err(|_| CompilerError::TooManyConstants)?;
                self.chunk.add(OpCode::Constant(constant), token.line);
            },
            Expr::Boolean(token, value) => {
                self.chunk.add(if value { OpCode::True } else { OpCode::False }, token.line);
            },
            Expr::Nil(token) => {
                self.chunk.add(OpCode::Nil, token.line);
            },
        }

        Ok(())
    }

    fn add_string(&mut self, s: String) -> Result<u8, CompilerError> {
        let object = Rc::new(Object::String(s));
        let constant = self.chunk.add_constant(Value::Object(object)).map_err(|_| CompilerError::TooManyConstants)?;

        Ok(constant)
    }
}