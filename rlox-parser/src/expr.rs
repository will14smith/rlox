use rlox_scanner::SourceToken;

#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Assign(SourceToken, Box<Expr>),
    Binary(Box<Expr>, SourceToken, Box<Expr>),
    Call(Box<Expr>, SourceToken, Vec<Expr>),
    Logical(Box<Expr>, SourceToken, Box<Expr>),
    Unary(SourceToken, Box<Expr>),
    Grouping(Box<Expr>),

    Var(SourceToken),
    String(String),
    Number(f64),
    Boolean(bool),
    Nil,
}