use rlox_scanner::{ SourceToken, Token };
use crate::Expr;

pub struct Parser {
    tokens: Vec<SourceToken>,

    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<SourceToken>) -> Parser {
        Parser {
            tokens,

            current: 0,
        }
    }

    pub fn parse(&mut self) -> Expr {
        self.expression()
    }

    // expressions
    fn expression(&mut self) -> Expr {
        self.equality()
    }

    fn equality(&mut self) -> Expr {
        let mut expr = self.comparison();

        while self.expect_one_of(vec![Token::BangEqual, Token::EqualEqual]) {
            let operator = self.previous().clone();
            let right = self.comparison();

            expr = Expr::Binary(Box::new(expr), operator, Box::new(right));
        }

        expr
    }

    fn comparison(&mut self) -> Expr {
        let mut expr = self.addition();

        while self.expect_one_of(vec![Token::Greater, Token::GreaterEqual, Token::Less, Token::LessEqual]) {
            let operator = self.previous().clone();
            let right = self.addition();

            expr = Expr::Binary(Box::new(expr), operator, Box::new(right));
        }

        expr
    }

    fn addition(&mut self) -> Expr {
        let mut expr = self.multiplication();

        while self.expect_one_of(vec![Token::Minus, Token::Plus]) {
            let operator = self.previous().clone();
            let right = self.multiplication();

            expr = Expr::Binary(Box::new(expr), operator, Box::new(right));
        }

        expr
    }

    fn multiplication(&mut self) -> Expr {
        let mut expr = self.unary();

        while self.expect_one_of(vec![Token::Slash, Token::Star]) {
            let operator = self.previous().clone();
            let right = self.unary();

            expr = Expr::Binary(Box::new(expr), operator, Box::new(right));
        }

        expr
    }

    fn unary(&mut self) -> Expr {
        if self.expect_one_of(vec![Token::Bang, Token::Minus]) {
            let operator = self.previous().clone();
            let right = self.unary();

            Expr::Unary(operator, Box::new(right))
        } else {
            self.primary()
        }
    }

    fn primary(&mut self) -> Expr {
        let token = self.advance();

        match &token.token {
            Token::False => Expr::Boolean(false),
            Token::True => Expr::Boolean(true),
            Token::Nil => Expr::Nil,

            Token::Number(val) => Expr::Number(*val),
            Token::String(val) => Expr::String(val.clone()),

            Token::LeftParen => {
                let expr = self.expression();
                self.consume(Token::RightParen, "Expected ')' after expression.");

                Expr::Grouping(Box::new(expr))
            }

            _ => unimplemented!(),
        }
    }

    // movement
    fn expect(&mut self, token: Token) -> bool {
        if self.check(token) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn expect_one_of(&mut self, tokens: Vec<Token>) -> bool {
        for token in tokens {
            if self.expect(token) {
                return true
            }
        }

        false
    }

    fn advance(&mut self) -> &SourceToken {
        if !self.is_at_end() {
            self.current += 1
        }

        self.previous()
    }

    fn consume(&mut self, expected: Token, error_message: &str)  {
        if !self.expect(expected) {
            // TODO handle error
        }
    }

    // checks
    fn check(&self, token: Token) -> bool {
        if self.is_at_end() {
            false
        } else {
            self.peek().token == token
        }
    }

    fn is_at_end(&self) -> bool {
        if self.current >= self.tokens.len() {
            return true
        }

        self.peek().token == Token::Eof
    }

    fn peek(&self) -> &SourceToken {
        self.tokens.get(self.current).unwrap()
    }

    fn previous(&self) -> &SourceToken {
        self.tokens.get(self.current - 1).unwrap()
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_expression(tokens: Vec<Token>) -> Expr {
        let source_tokens = tokens.into_iter()
            .map(tok_to_src)
            .collect();

        let mut parser = Parser::new(source_tokens);
        parser.expression()
    }

    fn tok_to_src(t: Token) -> SourceToken {
        SourceToken {
            token: t,
            lexeme: String::new(),
            line: 0
        }
    }

    #[test]
    fn test_primary() {
        assert_eq!(parse_expression(vec![Token::Nil]), Expr::Nil);
        assert_eq!(parse_expression(vec![Token::True]), Expr::Boolean(true));
        assert_eq!(parse_expression(vec![Token::False]), Expr::Boolean(false));

        assert_eq!(parse_expression(vec![Token::Number(123f64)]), Expr::Number(123f64));
        assert_eq!(parse_expression(vec![Token::String("abc".into())]), Expr::String("abc".into()));

        assert_eq!(parse_expression(vec![Token::LeftParen, Token::False, Token::RightParen]), Expr::Grouping(Box::new(Expr::Boolean(false))));
    }

    #[test]
    fn test_unary() {
        assert_eq!(parse_expression(vec![Token::Bang, Token::False]), Expr::Unary(tok_to_src(Token::Bang), Box::new(Expr::Boolean(false))));
        assert_eq!(parse_expression(vec![Token::Minus, Token::Number(123f64)]), Expr::Unary(tok_to_src(Token::Minus), Box::new(Expr::Number(123f64))));
    }

    #[test]
    fn test_binary() {
        for operator in vec![Token::Slash, Token::Star, Token::Minus, Token::Plus, Token::Greater, Token::GreaterEqual, Token::Less, Token::LessEqual, Token::BangEqual, Token::EqualEqual] {
            assert_eq!(parse_expression(vec![Token::Number(123f64), operator.clone(), Token::Number(456f64)]), Expr::Binary(Box::new(Expr::Number(123f64)), tok_to_src(operator), Box::new(Expr::Number(456f64))));
        }
        for operator in vec![Token::Slash, Token::Star, Token::Minus, Token::Plus, Token::Greater, Token::GreaterEqual, Token::Less, Token::LessEqual, Token::BangEqual, Token::EqualEqual] {
            assert_eq!(parse_expression(vec![Token::Number(123f64), operator.clone(), Token::Number(456f64), operator.clone(), Token::Number(789f64)]),
                       Expr::Binary(Box::new(Expr::Binary(Box::new(Expr::Number(123f64)), tok_to_src(operator.clone()), Box::new(Expr::Number(456f64)))), tok_to_src(operator), Box::new(Expr::Number(789f64))));
        }

        assert_eq!(parse_expression(vec![Token::Number(123f64), Token::Plus, Token::Number(456f64), Token::Star, Token::Number(789f64)]),
                   Expr::Binary(Box::new(Expr::Number(123f64)), tok_to_src(Token::Plus), Box::new(Expr::Binary(Box::new(Expr::Number(456f64)), tok_to_src(Token::Star), Box::new(Expr::Number(789f64))))));
    }
}