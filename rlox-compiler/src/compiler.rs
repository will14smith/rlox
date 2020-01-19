use std::rc::Rc;
use rlox_scanner::Token;
use rlox_parser::{Expr, Stmt};
use crate::chunk::ChunkReference;
use crate::{Chunk, Object, OpCode, Value};
use crate::op::OpCode::JumpIfFalse;

pub struct Compiler<'a> {
    chunk: &'a mut Chunk,

    locals: Vec<Local>,
    scope_depth: u8,
}

pub struct Local {
    pub name: String,
    pub scope_depth: u8,
}

#[derive(Debug)]
pub enum CompilerError {
    TooManyConstants,
    TooManyLocals,
    VariableAlreadyDeclared(String),
}

impl<'a> Compiler<'a> {
    pub fn new(chunk: &'a mut Chunk) -> Compiler<'a> {
        Compiler {
            chunk,

            locals: Vec::new(),
            scope_depth: 0,
        }
    }
}

type JumpOpFactory = Box<dyn Fn(i16) -> OpCode>;

struct JumpPatchReference {
    chunk_ref: ChunkReference,
    offset: usize,
    op_factory: JumpOpFactory,
}

struct JumpLoopReference {
    offset: usize,
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
            Stmt::Block(stmts) => {
                self.begin_scope();
                for stmt in stmts {
                    self.compile_stmt(stmt)?;
                }
                self.end_scope();
            },
            Stmt::Class(_, _) => unimplemented!(),
            Stmt::Expression(expr) => {
                self.compile_expr(expr)?;
                self.chunk.add(OpCode::Pop, 0); // TODO get line
            },
            Stmt::Function(_) => unimplemented!(),
            Stmt::If(cond, true_branch, false_branch) => {
                self.compile_expr(cond)?;

                let false_jump = self.jump(Box::new(OpCode::JumpIfFalse));

                self.chunk.add(OpCode::Pop, 0); // TODO line number
                self.compile_stmt(*true_branch)?;

                match false_branch {
                    Some(false_branch) => {
                        let true_jump = self.jump(Box::new(OpCode::Jump));

                        self.resolve_jump(&false_jump);
                        self.chunk.add(OpCode::Pop, 0); // TODO line number
                        self.compile_stmt(*false_branch)?;
                        self.resolve_jump(&true_jump);
                    },
                    None => {
                        self.resolve_jump(&false_jump);
                    },
                }
            }
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

                if self.scope_depth > 0 {
                    if self.locals.len() == std::u8::MAX as usize {
                        return Err(CompilerError::TooManyLocals);
                    }

                    let existing_in_scope = self.locals.iter().any(|x| x.scope_depth == self.scope_depth && x.name == name.lexeme);
                    if existing_in_scope {
                        return Err(CompilerError::VariableAlreadyDeclared(name.lexeme));
                    }

                    self.locals.push(Local {
                        name: name.lexeme,
                        scope_depth: self.scope_depth,
                    });
                } else {
                    let constant = self.add_string(name.lexeme)?;
                    self.chunk.add(OpCode::DefineGlobal(constant), 0);
                }
            },
            Stmt::While(condition, body) => {
                let loop_start = self.loop_start();
                self.compile_expr(condition)?;
                let exit_jump = self.jump(Box::new(OpCode::JumpIfFalse));

                self.chunk.add(OpCode::Pop, 0); // TODO line number
                self.compile_stmt(*body)?;
                self.jump_loop(&loop_start, Box::new(OpCode::Jump));

                self.resolve_jump(&exit_jump);
                self.chunk.add(OpCode::Pop, 0); // TODO line number
            },
        }

        Ok(())
    }
    
    fn compile_expr(&mut self, expr: Expr) -> Result<(), CompilerError> {
        match expr {
            Expr::Assign(name, value) => {
                self.compile_expr(*value)?;

                match self.resolve_local(&name.lexeme) {
                    Some(local) => {
                        self.chunk.add(OpCode::SetLocal(local), name.line);
                    },
                    None => {
                        let constant = self.add_string(name.lexeme)?;
                        self.chunk.add(OpCode::SetGlobal(constant), name.line);
                    }
                }
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

                    _ => { panic!("Invalid binary operation {:?}", op.token); },
                };
            },
            Expr::Call(_, _, _) => unimplemented!(),
            Expr::Logical(left, op, right) => {
                self.compile_expr(*left)?;

                match &op.token {
                    Token::Or => {
                        let else_jump = self.jump(Box::new(OpCode::JumpIfFalse));
                        let end_jump = self.jump(Box::new(OpCode::Jump));

                        self.resolve_jump(&else_jump);
                        self.chunk.add(OpCode::Pop, op.line);
                        self.compile_expr(*right)?;

                        self.resolve_jump(&end_jump);
                    },
                    Token::And => {
                        let jump = self.jump(Box::new(OpCode::JumpIfFalse));

                        self.chunk.add(OpCode::Pop, op.line);
                        self.compile_expr(*right)?;

                        self.resolve_jump(&jump);
                    },

                    _ => panic!("Invalid logical operation {:?}", op.token)
                }
            },
            Expr::Unary(op, value) => {
                self.compile_expr(*value)?;

                match &op.token {
                    Token::Bang => self.chunk.add(OpCode::Not, op.line),
                    Token::Minus => self.chunk.add(OpCode::Negate, op.line),

                    _ => { panic!("Invalid unary operation {:?}", op.token); },
                };
            },
            Expr::Grouping(expr) => self.compile_expr(*expr)?,
            Expr::Var(name) => {
                match self.resolve_local(&name.lexeme) {
                    Some(local) => {
                        self.chunk.add(OpCode::GetLocal(local), name.line);
                    },
                    None => {
                        let constant = self.add_string(name.lexeme)?;
                        self.chunk.add(OpCode::GetGlobal(constant), name.line);
                    }
                }
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

    fn jump(&mut self, op_factory: JumpOpFactory) -> JumpPatchReference {
        let offset = self.chunk.len();
        let chunk_ref = self.chunk.add(OpCode::Jump(0), 0); // TODO line number?

        // TODO track unresolved jumps

        JumpPatchReference { chunk_ref, offset, op_factory }
    }
    fn resolve_jump(&mut self, jump: &JumpPatchReference) {
        let offset = self.chunk.len() - jump.offset;
        let op = (jump.op_factory)(offset as i16);
        self.chunk.patch(&jump.chunk_ref, op);
    }
    fn loop_start(&self) -> JumpLoopReference {
        JumpLoopReference { offset: self.chunk.len() }
    }
    fn jump_loop(&mut self, jump: &JumpLoopReference, op_factory: JumpOpFactory) {
        let offset = -((self.chunk.len() - jump.offset) as i16);

        self.chunk.add(op_factory(offset), 0); // TODO line number?
    }

    fn resolve_local(&mut self, name: &String) -> Option<u8> {
        self.locals.iter().enumerate().rev().find(|(_, local)| &local.name == name).map(|(i, _)| i as u8)
    }

    fn begin_scope(&mut self) {
        if self.scope_depth == std::u8::MAX {
            panic!("begin scope will overflow scope depth")
        }

        self.scope_depth += 1;
    }
    fn end_scope(&mut self) {
        if self.scope_depth == std::u8::MIN {
            panic!("ending scope without an open one")
        }

        self.scope_depth -= 1;

        while !self.locals.is_empty() && self.locals.last().unwrap().scope_depth > self.scope_depth {
            self.chunk.add(OpCode::Pop, 0); // TODO line number?
            self.locals.pop();
        }
    }
}