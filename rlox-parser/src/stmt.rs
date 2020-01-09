use std::rc::Rc;
use rlox_scanner::SourceToken;
use crate::Expr;

#[derive(Debug, PartialEq)]
pub enum Stmt {
    Expression(Expr),
    Function(Func),
    If(Expr, Box<Stmt>, Option<Box<Stmt>>),
    Print(Expr),
    Var(SourceToken, Option<Expr>),
    While(Expr, Box<Stmt>),
    Block(Vec<Stmt>),
}

#[derive(Debug, PartialEq)]
pub struct Func {
    pub name: SourceToken,
    pub parameters: Vec<SourceToken>,
    pub body: Rc<Stmt>,
}

impl Func {
    pub fn new(name: SourceToken, parameters: Vec<SourceToken>, body: Rc<Stmt>) -> Func {
        Func {
            name,
            parameters,
            body,
        }
    }
}