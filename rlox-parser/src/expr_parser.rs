use rlox_scanner::{ Token, SourceToken };
use crate::parser::{ Parser, ParserErrorDescription, ParserResult };
use crate::{ Expr };

pub struct ExprParser<'a> {
    parser: &'a mut Parser,
}

impl<'a> ExprParser<'a> {
    pub fn new(parser: &'a mut Parser) -> ExprParser<'a> {
        ExprParser {
            parser
        }
    }

    pub fn parse(&mut self) -> ParserResult<Expr> {
        self.assignment()
    }

    fn assignment(&mut self) -> ParserResult<Expr> {
        let expr = self.or()?;

        if self.parser.try_consume(Token::Equal) {
            let equals = self.parser.previous().clone();
            let value = self.assignment()?;

            match expr {
                Expr::Var(target) => {
                    Ok(Expr::Assign(target, Box::new(value)))
                }
                _ => {
                    Err(self.parser.error(&equals, ParserErrorDescription::InvalidAssignmentTarget))
                }
            }

        } else {
            Ok(expr)
        }
    }

    fn or(&mut self) -> ParserResult<Expr> {
        let mut expr = self.and()?;

        while self.parser.try_consume(Token::Or) {
            let operator = self.parser.previous().clone();
            let right = self.and()?;

            expr = Expr::Logical(Box::new(expr), operator, Box::new(right));
        }

        Ok(expr)
    }

    fn and(&mut self) -> ParserResult<Expr> {
        let mut expr = self.equality()?;

        while self.parser.try_consume(Token::And) {
            let operator = self.parser.previous().clone();
            let right = self.equality()?;

            expr = Expr::Logical(Box::new(expr), operator, Box::new(right));
        }

        Ok(expr)
    }

    fn equality(&mut self) -> ParserResult<Expr> {
        let mut expr = self.comparison()?;

        while self.parser.try_consume_one_of(vec![Token::BangEqual, Token::EqualEqual]) {
            let operator = self.parser.previous().clone();
            let right = self.comparison()?;

            expr = Expr::Binary(Box::new(expr), operator, Box::new(right));
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> ParserResult<Expr> {
        let mut expr = self.addition()?;

        while self.parser.try_consume_one_of(vec![Token::Greater, Token::GreaterEqual, Token::Less, Token::LessEqual]) {
            let operator = self.parser.previous().clone();
            let right = self.addition()?;

            expr = Expr::Binary(Box::new(expr), operator, Box::new(right));
        }

        Ok(expr)
    }

    fn addition(&mut self) -> ParserResult<Expr> {
        let mut expr = self.multiplication()?;

        while self.parser.try_consume_one_of(vec![Token::Minus, Token::Plus]) {
            let operator = self.parser.previous().clone();
            let right = self.multiplication()?;

            expr = Expr::Binary(Box::new(expr), operator, Box::new(right));
        }

        Ok(expr)
    }

    fn multiplication(&mut self) -> ParserResult<Expr> {
        let mut expr = self.unary()?;

        while self.parser.try_consume_one_of(vec![Token::Slash, Token::Star]) {
            let operator = self.parser.previous().clone();
            let right = self.unary()?;

            expr = Expr::Binary(Box::new(expr), operator, Box::new(right));
        }

        Ok(expr)
    }

    fn unary(&mut self) -> ParserResult<Expr> {
        if self.parser.try_consume_one_of(vec![Token::Bang, Token::Minus]) {
            let operator = self.parser.previous().clone();
            let right = self.unary()?;

            Ok(Expr::Unary(operator, Box::new(right)))
        } else {
            self.call()
        }
    }

    fn call(&mut self) -> ParserResult<Expr> {
        let mut expr = self.primary()?;

        while self.parser.try_consume(Token::LeftParen) {
            expr = self.finish_call(expr)?;
        }

        Ok(expr)
    }

    fn finish_call(&mut self, callee: Expr) -> ParserResult<Expr> {
        // left paren is already consumed
        let mut arguments = Vec::new();

        if !self.parser.check(Token::RightParen) {
            arguments.push(self.parse()?);
            while self.parser.try_consume(Token::Comma) {
                if arguments.len() >= 255 {
                    return Err(self.parser.error(self.parser.peek(), ParserErrorDescription::TooManyArguments));
                }

                arguments.push(self.parse()?);
            }
        }

        let paren = self.parser.consume(Token::RightParen, ParserErrorDescription::ExpectedToken(Token::RightParen, "Expected ')' after arguments".into()))?;

        Ok(Expr::Call(Box::new(callee), paren.clone(), arguments))
    }

    fn primary(&mut self) -> ParserResult<Expr> {
        let token = self.parser.advance();

        match &token.token {
            Token::False => Ok(Expr::Boolean(token.clone(), false)),
            Token::True => Ok(Expr::Boolean(token.clone(), true)),
            Token::Nil => Ok(Expr::Nil),

            Token::Number(val) => Ok(Expr::Number(token.clone(), *val)),
            Token::String(val) => Ok(Expr::String(token.clone(), val.clone())),

            Token::Identifier(_) => Ok(Expr::Var(token.clone())),

            Token::LeftParen => {
                if self.parser.is_at_end() {
                    return Err(self.parser.error(self.parser.peek(), ParserErrorDescription::ExpectedExpression));
                }

                let expr = self.parse()?;
                self.parser.consume(Token::RightParen, ParserErrorDescription::ExpectedToken(Token::RightParen, "Expected ')' after expression".into()))?;

                Ok(Expr::Grouping(Box::new(expr)))
            }

            _ => Err(self.parser.error(self.parser.peek(), ParserErrorDescription::ExpectedExpression)),
        }
    }
}

#[cfg(test)]
mod tests {
    use rlox_scanner::SourceToken;
    use super::*;

    fn parse_expression(tokens: Vec<Token>) -> ParserResult<Expr> {
        let mut source_tokens: Vec<SourceToken> = tokens.into_iter()
            .map(tok_to_src)
            .collect();
        source_tokens.push(tok_to_src(Token::Eof));

        let mut parser = Parser::new(source_tokens);
        let mut expr_parser = ExprParser::new(&mut parser);

        expr_parser.parse()
    }
    fn expect_parse_expression(tokens: Vec<Token>) -> Expr {
        parse_expression(tokens).expect("Failed to parse expression")
    }

    fn tok_to_src(t: Token) -> SourceToken {
        SourceToken {
            token: t.clone(),
            lexeme: format!("{:?}", t),
            line: 0
        }
    }

    fn ident(s: &str) -> Token {
        Token::Identifier(s.into())
    }

    fn expr_num(n: f64) -> Expr {
        Expr::Number(tok_to_src(Token::Number(n)), n)
    }
    fn expr_bool(b: bool) -> Expr {
        Expr::Boolean(tok_to_src(if b { Token::True } else { Token::False }), b)
    }
    fn expr_str(s: &str) -> Expr {
        Expr::String(tok_to_src(Token::String(s.into())), s.into())
    }

    #[test]
    fn test_primary() {
        assert_eq!(expect_parse_expression(vec![Token::Nil]), Expr::Nil);
        assert_eq!(expect_parse_expression(vec![Token::True]), expr_bool(true));
        assert_eq!(expect_parse_expression(vec![Token::False]), expr_bool(false));

        assert_eq!(expect_parse_expression(vec![Token::Number(123f64)]), expr_num(123f64));
        assert_eq!(expect_parse_expression(vec![Token::String("abc".into())]), expr_str("abc"));

        assert_eq!(expect_parse_expression(vec![ident("abc")]), Expr::Var(tok_to_src(ident("abc"))));

        assert_eq!(expect_parse_expression(vec![Token::LeftParen, Token::False, Token::RightParen]), Expr::Grouping(Box::new(expr_bool(false))));
    }

    #[test]
    fn test_unary() {
        assert_eq!(expect_parse_expression(vec![Token::Bang, Token::False]), Expr::Unary(tok_to_src(Token::Bang), Box::new(expr_bool(false))));
        assert_eq!(expect_parse_expression(vec![Token::Minus, Token::Number(123f64)]), Expr::Unary(tok_to_src(Token::Minus), Box::new(expr_num(123f64))));
    }

    #[test]
    fn test_binary() {
        for operator in vec![Token::Slash, Token::Star, Token::Minus, Token::Plus, Token::Greater, Token::GreaterEqual, Token::Less, Token::LessEqual, Token::BangEqual, Token::EqualEqual] {
            assert_eq!(expect_parse_expression(vec![Token::Number(123f64), operator.clone(), Token::Number(456f64)]),
                       Expr::Binary(Box::new(expr_num(123f64)), tok_to_src(operator.clone()), Box::new(expr_num(456f64))));
            assert_eq!(expect_parse_expression(vec![Token::Number(123f64), operator.clone(), Token::Number(456f64), operator.clone(), Token::Number(789f64)]),
                       Expr::Binary(Box::new(Expr::Binary(Box::new(expr_num(123f64)), tok_to_src(operator.clone()), Box::new(expr_num(456f64)))), tok_to_src(operator), Box::new(expr_num(789f64))));
        }

        assert_eq!(expect_parse_expression(vec![Token::Number(123f64), Token::Plus, Token::Number(456f64), Token::Star, Token::Number(789f64)]),
                   Expr::Binary(Box::new(expr_num(123f64)), tok_to_src(Token::Plus), Box::new(Expr::Binary(Box::new(expr_num(456f64)), tok_to_src(Token::Star), Box::new(expr_num(789f64))))));
    }

    #[test]
    fn test_logical() {
        for operator in vec![Token::And, Token::Or] {
            assert_eq!(expect_parse_expression(vec![Token::False, operator.clone(), Token::True]),
                       Expr::Logical(Box::new(expr_bool(false)), tok_to_src(operator.clone()), Box::new(expr_bool(true))));
            assert_eq!(expect_parse_expression(vec![Token::False, operator.clone(), Token::True, operator.clone(), Token::True]),
                       Expr::Logical(Box::new(Expr::Logical(Box::new(expr_bool(false)), tok_to_src(operator.clone()), Box::new(expr_bool(true)))), tok_to_src(operator), Box::new(expr_bool(true))));
        }
    }

    #[test]
    fn test_call() {
        assert_eq!(expect_parse_expression(vec![ident("abc"), Token::LeftParen, Token::RightParen]), Expr::Call(Box::new(Expr::Var(tok_to_src(ident("abc")))), tok_to_src(Token::RightParen), vec![]));
        assert_eq!(expect_parse_expression(vec![ident("abc"), Token::LeftParen, Token::Number(123f64), Token::RightParen]), Expr::Call(Box::new(Expr::Var(tok_to_src(ident("abc")))), tok_to_src(Token::RightParen), vec![expr_num(123f64)]));
        assert_eq!(expect_parse_expression(vec![ident("abc"), Token::LeftParen, Token::Number(123f64), Token::Comma, Token::Number(456f64), Token::RightParen]), Expr::Call(Box::new(Expr::Var(tok_to_src(ident("abc")))), tok_to_src(Token::RightParen), vec![expr_num(123f64), expr_num(456f64)]));
    }

    #[test]
    fn test_assignment() {
        assert_eq!(expect_parse_expression(vec![ident("abc"), Token::Equal, Token::Number(123f64)]), Expr::Assign(tok_to_src(ident("abc")), Box::new(expr_num(123f64))));
        assert_eq!(expect_parse_expression(vec![ident("abc"), Token::Equal, ident("def"), Token::Equal, Token::Number(123f64)]), Expr::Assign(tok_to_src(ident("abc")), Box::new(Expr::Assign(tok_to_src(ident("def")), Box::new(expr_num(123f64))))));
    }

    #[test]
    fn test_error() {
        let result = parse_expression(vec![Token::LeftParen, Token::False]);
        assert!(result.is_err());

        let result = parse_expression(vec![Token::LeftParen]);
        assert!(result.is_err());

        let result = parse_expression(vec![Token::Number(123f64), Token::Equal, Token::Number(123f64)]);
        assert!(result.is_err());
    }
}