use crate::{ Token, SourceToken };

pub struct Scanner<'a> {
    source: &'a str,
}

pub struct ScannerIterator<'a> {
    source: &'a [u8],

    start: usize,
    current: usize,

    line: u32,
    line_offset: u32,
}

type ScanResult = Result<SourceToken, ScannerError>;

#[derive(Debug)]
pub struct ScannerError {
    pub error: ScannerErrorType,

    pub start: usize,
    pub current: usize,

    pub line: u32,
    pub line_offset: u32,
}
#[derive(Debug)]
pub enum ScannerErrorType {
    UnknownCharacter(u8),
    Utf8Error(::std::str::Utf8Error)
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Scanner {
        Scanner {
            source,
        }
    }

    pub fn tokens(&self) -> ScannerIterator {
        ScannerIterator {
            source: self.source.as_bytes(),

            start: 0,
            current: 0,

            line: 1,
            line_offset: 0,
        }
    }
}

impl<'a> ScannerIterator<'a> {
    fn scan_token(&mut self) -> ScanResult {
        if self.is_at_end() {
            let token = self.token(Token::Eof);
            self.current += 1;
            return token
        }


        let c = self.advance();

        match c {
            0x28 => self.token(Token::LeftParen),
            0x29 => self.token(Token::RightParen),
            0x7B => self.token(Token::LeftBrace),
            0x7D => self.token(Token::RightBrace),
            0x2C => self.token(Token::Comma),
            0x2E => self.token(Token::Dot),
            0x2D => self.token(Token::Minus),
            0x2B => self.token(Token::Plus),
            0x3B => self.token(Token::Semicolon),
            0x2A => self.token(Token::Star),

            _ => Err(self.error(ScannerErrorType::UnknownCharacter(c)))
        }
    }

    fn token(&self, token: Token) -> ScanResult {
        let lexeme = ::std::str::from_utf8(&self.source[self.start..self.current])
            .map_err(|e| self.error(ScannerErrorType::Utf8Error(e)))?;

        Ok(SourceToken {
            token,
            lexeme: lexeme.into(),

            line: self.line,
            line_offset: self.line_offset
        })
    }

    fn error(&self, error: ScannerErrorType) -> ScannerError {
        ScannerError {
            error,

            start: self.start,
            current: self.current,

            line: self.line,
            line_offset: self.line_offset
        }
    }

    fn advance(&mut self) -> u8 {
        self.current += 1;
        self.source[self.current - 1]
    }

    fn is_past_end(&self) -> bool {
        self.current > self.source.len()
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }
}

impl<'a> Iterator for ScannerIterator<'a> {
    type Item = ScanResult;

    fn next(&mut self) -> Option<Self::Item> {
        if self.is_past_end() {
            None
        } else {
            self.start = self.current;
            Some(self.scan_token())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(source: &str) -> Vec<ScanResult> {
        let scanner = Scanner::new(source);
        let tokens =  scanner.tokens();

        tokens.collect()
    }

    fn get_token(source: &str, index: usize) -> ScanResult {
        let mut tokens = parse(source);
        assert!(index < tokens.len(), "Tried to get token at index {} but there was only {} tokens", index, tokens.len());
        tokens.remove(index)
    }

    #[test]
    fn test_parse_single_char() -> Result<(), ScannerError> {
        assert_eq!(get_token("(", 0)?.token, Token::LeftParen);
        assert_eq!(get_token(")", 0)?.token, Token::RightParen);
        assert_eq!(get_token("{", 0)?.token, Token::LeftBrace);
        assert_eq!(get_token("}", 0)?.token, Token::RightBrace);
        assert_eq!(get_token(",", 0)?.token, Token::Comma);
        assert_eq!(get_token(".", 0)?.token, Token::Dot);
        assert_eq!(get_token("-", 0)?.token, Token::Minus);
        assert_eq!(get_token("+", 0)?.token, Token::Plus);
        assert_eq!(get_token(";", 0)?.token, Token::Semicolon);
        assert_eq!(get_token("*", 0)?.token, Token::Star);

        Ok(())
    }

    #[test]
    fn test_parse_eof() -> Result<(), ScannerError> {
        assert_eq!(get_token("(+)", 3)?.token, Token::Eof);

        Ok(())
    }

    #[test]
    fn test_parse_invalid_char() {
        let result = get_token("@", 0);
        assert!(result.is_err());
    }
}