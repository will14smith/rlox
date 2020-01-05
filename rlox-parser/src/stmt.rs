use rlox_scanner::SourceToken;
use crate::Expr;

#[derive(Debug, PartialEq)]
pub enum Stmt {
    Expression(Expr),
    Print(Expr),
    Var(SourceToken, Option<Expr>),
    Block(Vec<Stmt>),
}