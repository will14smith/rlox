use crate::{ Token, SourceToken };

pub struct Scanner<'a> {
    source: &'a str,
}

pub struct ScannerIterator<'a> {
    source: &'a [u8],

    start: usize,
    current: usize,

    line: u32,
}

type ScanResult = Result<SourceToken, ScannerError>;

#[derive(Debug)]
pub struct ScannerError {
    pub error: ScannerErrorType,

    pub start: usize,
    pub current: usize,

    pub line: u32,
}
#[derive(Debug, PartialEq)]
pub enum ScannerErrorType {
    UnknownCharacter(u8),
    Utf8Error(::std::str::Utf8Error),
    UnterminatedString,
    InvalidNumber(::std::num::ParseFloatError)
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

            0x21 => if self.expect(0x3D) { self.token(Token::BangEqual) } else { self.token(Token::Bang) },
            0x3D => if self.expect(0x3D) { self.token(Token::EqualEqual) } else { self.token(Token::Equal) },
            0x3C => if self.expect(0x3D) { self.token(Token::LessEqual) } else { self.token(Token::Less) },
            0x3E => if self.expect(0x3D) { self.token(Token::GreaterEqual) } else { self.token(Token::Greater) },

            0x2F => {
                if self.expect(0x2F) {
                    while self.peek() != 0x0A && !self.is_at_end() { self.advance(); }
                    self.token(Token::Comment)
                } else {
                    self.token(Token::Slash)
                }
            }

            0x09 | 0x0D | 0x20 => {
                self.token(Token::Whitespace)
            }

            0x0A => {
                let token = self.token(Token::NewLine);
                self.line += 1;
                token
            }

            0x22 => self.string(),

            0x30..=0x39 => self.number(),

            0x41..=0x5A | 0x5F | 0x61..=0x7A => self.identifier(),

            _ => Err(self.error(ScannerErrorType::UnknownCharacter(c)))
        }
    }

    // tokens
    fn string(&mut self) -> ScanResult {
        // already consumed the opening "

        while self.peek() != 0x22 && !self.is_at_end() {
            if self.peek() == 0x0A { self.line += 1 }
            self.advance();
        }

        if self.is_at_end() {
            Err(self.error(ScannerErrorType::UnterminatedString))
        } else {
            // consume the closing "
            self.advance();

            let value = self.slice_source(self.start+1..self.current-1)?;
            self.token(Token::String(value.into()))
        }
    }

     fn number(&mut self) -> ScanResult {
         while is_digit(self.peek()) {
             self.advance();
         }

         if self.peek() == 0x2E && is_digit(self.peek_next()) {
             // consume .
             self.advance();

             while is_digit(self.peek()) {
                 self.advance();
             }
         }

         let str_value = self.slice_source(self.start..self.current)?;
         let value = str_value.parse::<f64>()
             .map_err(|e| self.error(ScannerErrorType::InvalidNumber(e)))?;

         self.token(Token::Number(value))
     }

    fn identifier(&mut self) -> ScanResult {
        while is_alphanumeric(self.peek()) {
            self.advance();
        }

        let value = self.slice_source(self.start..self.current)?;
        match identifier_to_keyword(value) {
            Some(token) => self.token(token),
            None => self.token(Token::Identifier(value.into()))
        }    }

    // results
    fn token(&self, token: Token) -> ScanResult {
        let lexeme = self.slice_source(self.start..self.current)?;

        Ok(SourceToken {
            token,
            lexeme: lexeme.into(),

            line: self.line,
        })
    }
    fn error(&self, error: ScannerErrorType) -> ScannerError {
        ScannerError {
            error,

            start: self.start,
            current: self.current,

            line: self.line,
        }
    }

    // movement
    fn peek(&self) -> u8 {
        if self.is_at_end() {
            0
        } else {
            self.source[self.current]
        }
    }
    fn peek_next(&self) -> u8 {
        if self.current + 1 >= self.source.len()  {
            0
        } else {
            self.source[self.current + 1]
        }
    }

    fn advance(&mut self) -> u8 {
        self.current += 1;
        self.source[self.current - 1]
    }

    fn expect(&mut self, expected: u8) -> bool {
        if self.is_at_end() {
            return false;
        }
        if self.source[self.current] != expected {
            return false;
        }

        self.current += 1;
        return true;
    }

    fn slice_source(&self, range: ::std::ops::Range<usize>) -> Result<&str, ScannerError> {
        ::std::str::from_utf8(&self.source[range])
            .map_err(|e| self.error(ScannerErrorType::Utf8Error(e)))
    }

    // checks
    fn is_past_end(&self) -> bool {
        self.current > self.source.len()
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }
}

fn is_digit(v: u8) -> bool {
    match v { 0x30..=0x39 => true, _ => false }
}
fn is_alpha(v: u8) -> bool {
    match v { 0x41..=0x5A | 0x5F | 0x61..=0x7A => true, _ => false }
}
fn is_alphanumeric(v: u8) -> bool {
    is_alpha(v) || is_digit(v)
}

fn identifier_to_keyword(identifier: &str) -> Option<Token> {
    match identifier {
        "and" => Some(Token::And),
        "class" => Some(Token::Class),
        "else" => Some(Token::Else),
        "false" => Some(Token::False),
        "for" => Some(Token::For),
        "fun" => Some(Token::Fun),
        "if" => Some(Token::If),
        "nil" => Some(Token::Nil),
        "or" => Some(Token::Or),
        "print" => Some(Token::Print),
        "return" => Some(Token::Return),
        "super" => Some(Token::Super),
        "this" => Some(Token::This),
        "true" => Some(Token::True),
        "var" => Some(Token::Var),
        "while" => Some(Token::While),

        _ => None
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

    fn assert_error(result: ScanResult, expected: ScannerErrorType) {
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().error, expected)

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

        assert_eq!(get_token("!", 0)?.token, Token::Bang);
        assert_eq!(get_token("=", 0)?.token, Token::Equal);
        assert_eq!(get_token("<", 0)?.token, Token::Less);
        assert_eq!(get_token(">", 0)?.token, Token::Greater);

        assert_eq!(get_token("/", 0)?.token, Token::Slash);

        Ok(())
    }

    #[test]
    fn test_parse_compound_char() -> Result<(), ScannerError> {
        assert_eq!(get_token("!(", 0)?.token, Token::Bang);
        assert_eq!(get_token("=(", 0)?.token, Token::Equal);
        assert_eq!(get_token("<(", 0)?.token, Token::Less);
        assert_eq!(get_token(">(", 0)?.token, Token::Greater);

        assert_eq!(get_token("!=", 0)?.token, Token::BangEqual);
        assert_eq!(get_token("==", 0)?.token, Token::EqualEqual);
        assert_eq!(get_token("<=", 0)?.token, Token::LessEqual);
        assert_eq!(get_token(">=", 0)?.token, Token::GreaterEqual);

        Ok(())
    }

    #[test]
    fn test_parse_comment() -> Result<(), ScannerError> {
        assert_eq!(get_token("/(", 0)?.token, Token::Slash);

        assert_eq!(get_token("// hello", 0)?.token, Token::Comment);
        assert_eq!(get_token("/// hello", 0)?.token, Token::Comment);
        assert_eq!(get_token("// hello\n/", 2)?.token, Token::Slash);

        Ok(())
    }

    #[test]
    fn test_parse_string() -> Result<(), ScannerError> {
        assert_eq!(get_token("\"\"", 0)?.token, Token::String("".into()));
        assert_eq!(get_token("\"abc\"", 0)?.token, Token::String("abc".into()));
        assert_eq!(get_token("\"ab\nc\"", 0)?.token, Token::String("ab\nc".into()));

        Ok(())
    }

    #[test]
    fn test_parse_number() -> Result<(), ScannerError> {
        assert_eq!(get_token("1", 0)?.token, Token::Number(1f64));
        assert_eq!(get_token("12", 0)?.token, Token::Number(12f64));
        assert_eq!(get_token("12.34", 0)?.token, Token::Number(12.34f64));

        Ok(())
    }

    #[test]
    fn test_parse_identifier() -> Result<(), ScannerError> {
        assert_eq!(get_token("a", 0)?.token, Token::Identifier("a".into()));
        assert_eq!(get_token("_", 0)?.token, Token::Identifier("_".into()));
        assert_eq!(get_token("_a", 0)?.token, Token::Identifier("_a".into()));
        assert_eq!(get_token("_0", 0)?.token, Token::Identifier("_0".into()));

        Ok(())
    }

    #[test]
    fn test_parse_keyword() -> Result<(), ScannerError> {
        assert_eq!(get_token("and", 0)?.token, Token::And);
        assert_eq!(get_token("class", 0)?.token, Token::Class);
        assert_eq!(get_token("else", 0)?.token, Token::Else);
        assert_eq!(get_token("false", 0)?.token, Token::False);
        assert_eq!(get_token("for", 0)?.token, Token::For);
        assert_eq!(get_token("fun", 0)?.token, Token::Fun);
        assert_eq!(get_token("if", 0)?.token, Token::If);
        assert_eq!(get_token("nil", 0)?.token, Token::Nil);
        assert_eq!(get_token("or", 0)?.token, Token::Or);
        assert_eq!(get_token("print", 0)?.token, Token::Print);
        assert_eq!(get_token("return", 0)?.token, Token::Return);
        assert_eq!(get_token("super", 0)?.token, Token::Super);
        assert_eq!(get_token("this", 0)?.token, Token::This);
        assert_eq!(get_token("true", 0)?.token, Token::True);
        assert_eq!(get_token("var", 0)?.token, Token::Var);
        assert_eq!(get_token("while", 0)?.token, Token::While);

        Ok(())
    }

    #[test]
    fn test_parse_new_line() -> Result<(), ScannerError> {
        assert_eq!(get_token("+\n+", 0)?.line, 1);
        assert_eq!(get_token("+\n+", 1)?.line, 1);
        assert_eq!(get_token("+\n+", 2)?.line, 2);

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
        assert_error(result, ScannerErrorType::UnknownCharacter(0x40));
    }

    #[test]
    fn test_parse_unterminated_string() {
        let result = get_token("\"abc", 0);
        assert_error(result, ScannerErrorType::UnterminatedString);
    }
}