use rlox_scanner::SourceToken;
use crate::Expr;

#[derive(Debug, PartialEq)]
pub enum Stmt {
    Expression(Expr),
    If(Expr, Box<Stmt>, Option<Box<Stmt>>),
    Print(Expr),
    Var(SourceToken, Option<Expr>),
    Block(Vec<Stmt>),
}