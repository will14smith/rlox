use rlox_scanner::SourceToken;
use crate::Expr;

#[derive(Clone, Debug, PartialEq)]
pub enum Stmt {
    Expression(Expr),
    Function(Func),
    If(Expr, Box<Stmt>, Option<Box<Stmt>>),
    Print(Expr),
    Return(SourceToken, Option<Expr>),
    Var(SourceToken, Option<Expr>),
    While(Expr, Box<Stmt>),
    Block(Vec<Stmt>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Func {
    pub name: SourceToken,
    pub parameters: Vec<SourceToken>,
    pub body: Vec<Stmt>,
}

impl Func {
    pub fn new(name: SourceToken, parameters: Vec<SourceToken>, body: Vec<Stmt>) -> Func {
        Func {
            name,
            parameters,
            body,
        }
    }
}