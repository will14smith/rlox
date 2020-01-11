#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    // Single-character tokens.
    LeftParen, RightParen, LeftBrace, RightBrace,
    Comma, Dot, Minus, Plus, Semicolon, Slash, Star,

    // One or two character tokens.
    Bang, BangEqual,
    Equal, EqualEqual,
    Greater, GreaterEqual,
    Less, LessEqual,

    // Literals.
    Identifier(String),
    String(String),
    Number(f64),

    // Keywords.
    And, Class, Else, False, Fun, For, If, Nil, Or,
    Print, Return, Super, This, True, Var, While,

    Comment, Whitespace, NewLine, Eof
}

#[derive(Clone, Debug, PartialEq)]
pub struct SourceToken {
    pub token: Token,
    pub lexeme: String,
    pub line: usize,
}

impl Default for SourceToken {
    fn default() -> Self {
        SourceToken {
            token: Token::Eof,
            lexeme: String::new(),
            line: 0
        }
    }
}