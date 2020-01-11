use std::mem::Discriminant;
use rlox_scanner::{ SourceToken, Token };

pub struct Parser {
    tokens: Vec<SourceToken>,

    current: usize,
}

#[derive(Debug, PartialEq)]
pub struct ParserError {
    pub line: usize,
    pub location: String,
    pub description: ParserErrorDescription,
}
#[derive(Debug, PartialEq)]
pub enum ParserErrorDescription {
    ExpectedToken(Token, String),
    ExpectedExpression,
    ExpectedIdentifier(String),
    InvalidAssignmentTarget,
    TooManyArguments,
    TooManyParameters,
}

pub type ParserResult<T> = Result<T, ParserError>;

impl Parser {
    pub fn new(tokens: Vec<SourceToken>) -> Parser {
        Parser {
            tokens,

            current: 0,
        }
    }

    // movement
    pub fn try_consume(&mut self, token: Token) -> bool {
        self.try_consume_discriminant(::std::mem::discriminant(&token))
    }
    pub fn try_consume_discriminant(&mut self, token: Discriminant<Token>) -> bool {
        if self.check_discriminant(token) {
            self.advance();
            true
        } else {
            false
        }
    }

    pub fn try_consume_one_of(&mut self, tokens: Vec<Token>) -> bool {
        for token in tokens {
            if self.try_consume(token) {
                return true
            }
        }

        false
    }

    pub fn consume(&mut self, expected: Token, error: ParserErrorDescription) -> ParserResult<&SourceToken> {
        self.consume_discriminant(::std::mem::discriminant(&expected), error)
    }
    pub fn consume_discriminant(&mut self, expected: Discriminant<Token>, error: ParserErrorDescription) -> ParserResult<&SourceToken> {
        if self.check_discriminant(expected) {
           Ok(self.advance())
        } else {
            Err(self.error(self.peek(), error))
        }
    }

    pub fn advance(&mut self) -> &SourceToken {
        if !self.is_at_end() {
            self.current += 1
        }

        self.previous()
    }

    pub fn error(&self, token: &SourceToken, description: ParserErrorDescription) -> ParserError {
        ParserError {
            line: token.line,
            location: if token.token == Token::Eof { "at end".into() } else { format!("at '{}'", token.lexeme) },
            description,
        }
    }

    // checks
    pub fn check(&self, token: Token) -> bool {
        self.check_discriminant(::std::mem::discriminant(&token))
    }
    pub fn check_discriminant(&self, token: Discriminant<Token>) -> bool {
        if self.is_at_end() {
            false
        } else {
            ::std::mem::discriminant(&self.peek().token) == token
        }
    }

    pub fn is_at_end(&self) -> bool {
        self.peek().token == Token::Eof
    }

    pub fn peek(&self) -> &SourceToken {
        self.tokens.get(self.current).unwrap()
    }

    pub fn previous(&self) -> &SourceToken {
        self.tokens.get(self.current - 1).unwrap()
    }

}