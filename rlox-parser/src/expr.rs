use rlox_scanner::SourceToken;

#[derive(Debug, PartialEq)]
pub enum Expr {
    Binary(Box<Expr>, SourceToken, Box<Expr>),
    Unary(SourceToken, Box<Expr>),
    Grouping(Box<Expr>),

    String(String),
    Number(f64),
    Boolean(bool),
    Nil,
}