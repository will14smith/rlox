use std::collections::HashMap;
use std::mem::Discriminant;
use rlox_scanner::{ Token, SourceToken };
use crate::parser::{ Parser, ParserErrorDescription, ParserResult };
use crate::{Expr, ParserError};

pub struct ExprParser<'a> {
    parser: &'a mut Parser,
    rules: HashMap<Discriminant<Token>, ParseRule<'a>>,
}

type PrefixFn<'a> = fn(&mut ExprParser<'a>) -> ParserResult<Expr>;
type InfixFn<'a> = fn(&mut ExprParser<'a>, Expr) -> ParserResult<Expr>;

struct ParseRule<'a> {
    prefix: Option<PrefixFn<'a>>,
    infix: Option<InfixFn<'a>>,
    precedence: Precedence,
}

impl<'a> ParseRule<'a> {
    pub fn new(prefix: Option<PrefixFn<'a>>, infix: Option<InfixFn<'a>>, precedence: Precedence) -> ParseRule<'a> {
        ParseRule {
            prefix,
            infix,
            precedence
        }
    }
    pub fn new_prefix(prefix: PrefixFn<'a>, precedence: Precedence) -> ParseRule<'a> {
        Self::new(Some(prefix), None, precedence)
    }
    pub fn new_infix(infix: InfixFn<'a>, precedence: Precedence) -> ParseRule<'a> {
        Self::new(None, Some(infix), precedence)
    }
}

#[derive(Copy, Clone, PartialOrd, PartialEq)]
#[repr(u8)]
enum Precedence {
    None = 0,
    Assignment,
    Or,
    And,
    Equality,
    Comparison,
    Term,
    Factor,
    Unary,
    Call,
    Primary
}

impl Precedence {
    pub fn next(self) -> Self {
        let val = unsafe { ::std::mem::transmute::<Self, u8>(self) } as u16 + 1;
        let max = unsafe { ::std::mem::transmute::<Self, u8>(Self::Primary) } as u16;

        if val >= max {
            Self::Primary
        } else {
            unsafe { ::std::mem::transmute(val as u8) }
        }
    }
}

impl<'a> ExprParser<'a> {
    pub fn new(parser: &'a mut Parser) -> ExprParser<'a> {
        let mut rules = HashMap::new();

        fn add_rule<'a>(rules: &mut HashMap<Discriminant<Token>, ParseRule<'a>>, t: Token, rule: ParseRule<'a>) {
            rules.insert(::std::mem::discriminant(&t), rule);
        }

        add_rule(&mut rules, Token::Eof, ParseRule::new(None, None, Precedence::None));
        add_rule(&mut rules, Token::LeftParen, ParseRule::new_prefix(ExprParser::grouping, Precedence::None));

        add_rule(&mut rules, Token::Identifier(String::new()), ParseRule::new_prefix(ExprParser::literal, Precedence::None));
        add_rule(&mut rules, Token::Number(0f64), ParseRule::new_prefix(ExprParser::literal, Precedence::None));
        add_rule(&mut rules, Token::String(String::new()), ParseRule::new_prefix(ExprParser::literal, Precedence::None));
        add_rule(&mut rules, Token::True, ParseRule::new_prefix(ExprParser::literal, Precedence::None));
        add_rule(&mut rules, Token::False, ParseRule::new_prefix(ExprParser::literal, Precedence::None));
        add_rule(&mut rules, Token::Nil, ParseRule::new_prefix(ExprParser::literal, Precedence::None));

        add_rule(&mut rules, Token::Bang, ParseRule::new_prefix(ExprParser::unary, Precedence::Unary));

        add_rule(&mut rules, Token::Plus, ParseRule::new_infix(ExprParser::binary, Precedence::Term));
        add_rule(&mut rules, Token::Minus, ParseRule::new(Some(ExprParser::unary), Some(ExprParser::binary), Precedence::Term));
        add_rule(&mut rules, Token::Star, ParseRule::new_infix(ExprParser::binary, Precedence::Factor));
        add_rule(&mut rules, Token::Slash, ParseRule::new_infix(ExprParser::binary, Precedence::Factor));
        add_rule(&mut rules, Token::BangEqual, ParseRule::new_infix(ExprParser::binary, Precedence::Equality));
        add_rule(&mut rules, Token::EqualEqual, ParseRule::new_infix(ExprParser::binary, Precedence::Equality));
        add_rule(&mut rules, Token::Greater, ParseRule::new_infix(ExprParser::binary, Precedence::Comparison));
        add_rule(&mut rules, Token::GreaterEqual, ParseRule::new_infix(ExprParser::binary, Precedence::Comparison));
        add_rule(&mut rules, Token::Less, ParseRule::new_infix(ExprParser::binary, Precedence::Comparison));
        add_rule(&mut rules, Token::LessEqual, ParseRule::new_infix(ExprParser::binary, Precedence::Comparison));

        add_rule(&mut rules, Token::And, ParseRule::new_infix(ExprParser::logical, Precedence::And));
        add_rule(&mut rules, Token::Or, ParseRule::new_infix(ExprParser::logical, Precedence::Or));

        ExprParser {
            parser,
            rules,
        }
    }

    pub fn parse(&mut self) -> ParserResult<Expr> {
        self.parse_precedence(Precedence::Assignment)
    }

    fn parse_precedence(&mut self, precedence: Precedence) -> ParserResult<Expr> {
        self.parser.advance();

        let prefix = self.prefix_rule(self.parser.previous())?;
        let mut expr = prefix(self)?;

        while precedence <= self.precedence(self.parser.peek()) {
            self.parser.advance();

            let prev = self.parser.previous();
            let infix = self.infix_rule(prev)?;
            expr = match infix {
                Some(infix) => infix(self, expr)?,
                None => panic!("invalid rule for {:?}", prev),
            }
        }

        Ok(expr)
    }

    fn binary(&mut self, left: Expr) -> ParserResult<Expr> {
        let op = self.parser.previous().clone();

        let precedence = self.precedence(&op);
        let right = self.parse_precedence(precedence.next())?;

        Ok(Expr::Binary(Box::new(left), op, Box::new(right)))
    }
    fn logical(&mut self, left: Expr) -> ParserResult<Expr> {
        let op = self.parser.previous().clone();

        let precedence = self.precedence(&op);
        let right = self.parse_precedence(precedence.next())?;

        Ok(Expr::Logical(Box::new(left), op, Box::new(right)))
    }

    fn unary(&mut self) -> ParserResult<Expr> {
        let op = self.parser.previous().clone();
        let expr = self.parse_precedence(Precedence::Unary)?;

        Ok(Expr::Unary(op, Box::new(expr)))
    }

    fn grouping(&mut self) -> ParserResult<Expr> {
        let expr = self.parse()?;
        self.parser.consume(Token::RightParen, ParserErrorDescription::ExpectedToken(Token::RightParen, "Expected ')' after expression".into()))?;

        Ok(Expr::Grouping(Box::new(expr)))
    }

    fn literal(&mut self) -> ParserResult<Expr> {
        let token = self.parser.previous();

        match &token.token {
            Token::Identifier(value) => Ok(Expr::Var(token.clone())),
            Token::Number(value) => Ok(Expr::Number(token.clone(), *value)),
            Token::String(value) => Ok(Expr::String(token.clone(), value.clone())),
            Token::True => Ok(Expr::Boolean(token.clone(), true)),
            Token::False => Ok(Expr::Boolean(token.clone(), false)),
            Token::Nil => Ok(Expr::Nil(token.clone())),

            _ => panic!("ExprParser::literal called with {:?} token", token),
        }
    }

    fn rule(&self, token: &SourceToken) -> ParserResult<&ParseRule<'a>> {
        match self.rules.get(&::std::mem::discriminant(&token.token)) {
            Some(rule) => Ok(rule),
            None => Err(self.parser.error(token, ParserErrorDescription::ExpectedExpression)),
        }
    }
    fn prefix_rule(&self, token: &SourceToken) -> ParserResult<&PrefixFn<'a>> {
        let rule = self.rule(token)?;
        match &rule.prefix {
            Some(prefix) => Ok(prefix),
            None => Err(self.parser.error(token, ParserErrorDescription::ExpectedExpression)),
        }
    }
    fn infix_rule(&self, token: &SourceToken) -> ParserResult<Option<InfixFn<'a>>> {
        let rule = self.rule(token)?;
         Ok(rule.infix.clone())
    }
    fn precedence(&self, token: &SourceToken) -> Precedence {
        self.rule(token).map(|rule| rule.precedence).unwrap_or(Precedence::None)
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
        assert_eq!(expect_parse_expression(vec![Token::Nil]), Expr::Nil(tok_to_src(Token::Nil)));
        assert_eq!(expect_parse_expression(vec![Token::True]), expr_bool(true));
        assert_eq!(expect_parse_expression(vec![Token::False]), expr_bool(false));
        assert_eq!(expect_parse_expression(vec![Token::Number(123f64)]), expr_num(123f64));
        assert_eq!(expect_parse_expression(vec![Token::String("abc".into())]), expr_str("abc"));
        assert_eq!(expect_parse_expression(vec![ident("abc")]), Expr::Var(tok_to_src(ident("abc"))));

        assert_eq!(expect_parse_expression(vec![Token::LeftParen, Token::False, Token::RightParen]), Expr::Grouping(Box::new(expr_bool(false))));

        // should leave the trailing content alone
        assert_eq!(expect_parse_expression(vec![Token::Number(123f64), Token::Semicolon]), expr_num(123f64));
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
                       Expr::Binary(Box::new(Expr::Binary(Box::new(expr_num(123f64)), tok_to_src(operator.clone()), Box::new(expr_num(456f64)))), tok_to_src(operator.clone()), Box::new(expr_num(789f64))));
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