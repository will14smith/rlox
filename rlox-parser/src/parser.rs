use std::{
    mem::Discriminant,
};
use rlox_scanner::{ SourceToken, Token };
use crate::{
    Expr,
    stmt::{ Func, Stmt },
};

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

type ParserResult<T> = Result<T, ParserError>;

impl Parser {
    pub fn new(tokens: Vec<SourceToken>) -> Parser {
        Parser {
            tokens,

            current: 0,
        }
    }

    pub fn parse(&mut self) -> Vec<ParserResult<Stmt>> {
        let mut statements = Vec::new();

        while !self.is_at_end() {
            statements.push(self.declaration());
        }

        statements
    }

    // statements
    fn declaration(&mut self) -> ParserResult<Stmt> {
        fn inner(parser: &mut Parser) -> ParserResult<Stmt> {
            if parser.try_consume(Token::Class) {
                parser.class_declaration()
            } else if parser.try_consume(Token::Fun) {
                parser.function("function").map(Stmt::Function)
            } else if parser.try_consume(Token::Var) {
                parser.var_declaration()
            } else {
                parser.statement()
            }
        }

        match inner(self) {
            Ok(stmt) => Ok(stmt),
            Err(err) => {
                self.synchronize();
                Err(err)
            },
        }
    }

    fn class_declaration(&mut self) -> ParserResult<Stmt> {
        // class keyword is already consumed
        let name = self.consume_discriminant(::std::mem::discriminant(&Token::Identifier(String::new())), ParserErrorDescription::ExpectedIdentifier("Expected class name".into()))?;
        let name = name.clone();

        self.consume(Token::LeftBrace, ParserErrorDescription::ExpectedToken(Token::LeftBrace, "Expected '{' before class body".into()))?;

        let mut functions = Vec::new();
        while !self.check(Token::RightBrace) && !self.is_at_end() {
            functions.push(self.function("method")?);
        }

        self.consume(Token::RightBrace, ParserErrorDescription::ExpectedToken(Token::RightBrace, "Expected '}' after class body".into()))?;

        Ok(Stmt::Class(name, functions))
    }

    fn var_declaration(&mut self) -> ParserResult<Stmt> {
        // var keyword is already consumed
        let name = self.consume_discriminant(::std::mem::discriminant(&Token::Identifier(String::new())), ParserErrorDescription::ExpectedIdentifier("Expected variable name".into()))?;
        let name = name.clone();

        let initializer = if self.try_consume(Token::Equal) {
            Some(self.expression()?)
        } else {
            None
        };

        self.consume(Token::Semicolon, ParserErrorDescription::ExpectedToken(Token::Semicolon, "Expected ';' after variable declaration".into()))?;

        Ok(Stmt::Var(name, initializer))
    }

    fn statement(&mut self) -> ParserResult<Stmt> {
        if self.try_consume(Token::For) {
            self.for_statement()
        } else if self.try_consume(Token::If) {
            self.if_statement()
        } else if self.try_consume(Token::Print) {
            self.print_statement()
        } else if self.try_consume(Token::Return) {
            self.return_statement()
        } else if self.try_consume(Token::While) {
            self.while_statement()
        } else if self.try_consume(Token::LeftBrace) {
            Ok(Stmt::Block(self.block()?))
        } else {
            self.expression_statement()
        }
    }

    fn for_statement(&mut self) -> ParserResult<Stmt> {
        // for keyword is already consumed
        self.consume(Token::LeftParen, ParserErrorDescription::ExpectedToken(Token::LeftParen, "Expected '(' after 'for'".into()))?;

        let initializer = if self.try_consume(Token::Semicolon) {
            None
        } else if self.try_consume(Token::Var) {
            Some(self.var_declaration()?)
        } else {
            Some(self.expression_statement()?)
        };

        let condition = if self.check(Token::Semicolon) {
            Expr::Boolean(self.previous().clone(), true)
        } else {
            self.expression()?
        };
        self.consume(Token::Semicolon, ParserErrorDescription::ExpectedToken(Token::Semicolon, "Expected ';' after for condition".into()))?;

        let update = if self.check(Token::RightParen) {
            None
        } else {
            Some(self.expression()?)
        };
        self.consume(Token::RightParen, ParserErrorDescription::ExpectedToken(Token::RightParen, "Expected ')' after for update".into()))?;

        let mut body = self.statement()?;

        if let Some(update) = update {
            body = Stmt::Block(vec![body, Stmt::Expression(update)]);
        }

        body = Stmt::While(condition, Box::new(body));

        if let Some(initializer) = initializer {
            body = Stmt::Block(vec![initializer, body]);
        }

        Ok(body)
    }

    fn if_statement(&mut self) -> ParserResult<Stmt> {
        // if keyword is already consumed
        self.consume(Token::LeftParen, ParserErrorDescription::ExpectedToken(Token::LeftParen, "Expected '(' after 'if'".into()))?;
        let condition = self.expression()?;
        self.consume(Token::RightParen, ParserErrorDescription::ExpectedToken(Token::RightParen, "Expected ')' after if condition".into()))?;

        let then_branch = Box::new(self.statement()?);
        let else_branch = if self.try_consume(Token::Else) {
            Some(Box::new(self.statement()?))
        } else {
            None
        };

        Ok(Stmt::If(condition, then_branch, else_branch))
    }

    fn print_statement(&mut self) -> ParserResult<Stmt> {
        // print keyword is already consumed
        let value = self.expression()?;

        self.consume(Token::Semicolon, ParserErrorDescription::ExpectedToken(Token::Semicolon, "Expected ';' after value".into()))?;

        Ok(Stmt::Print(value))
    }

    fn return_statement(&mut self) -> ParserResult<Stmt> {
        let token = self.previous().clone();

        let value = if self.check(Token::Semicolon) {
            None
        } else {
            Some(self.expression()?)
        };

        self.consume(Token::Semicolon, ParserErrorDescription::ExpectedToken(Token::Semicolon, "Expected ';' after return value".into()))?;

        Ok(Stmt::Return(token, value))
    }

    fn while_statement(&mut self) -> ParserResult<Stmt> {
        // while keyword is already consumed
        self.consume(Token::LeftParen, ParserErrorDescription::ExpectedToken(Token::LeftParen, "Expected '(' after 'if'".into()))?;
        let condition = self.expression()?;
        self.consume(Token::RightParen, ParserErrorDescription::ExpectedToken(Token::RightParen, "Expected ')' after if condition".into()))?;

        let body = Box::new(self.statement()?);

        Ok(Stmt::While(condition, body))
    }

        fn block(&mut self) -> ParserResult<Vec<Stmt>> {
        // left brace is already consumed
        let mut statements = Vec::new();

        while !self.check(Token::RightBrace) && !self.is_at_end() {
            statements.push(self.declaration()?);
        }

        self.consume(Token::RightBrace, ParserErrorDescription::ExpectedToken(Token::RightBrace, "Expected '}' after block".into()))?;

        Ok(statements)
    }

    fn expression_statement(&mut self) -> ParserResult<Stmt> {
        let value = self.expression()?;

        self.consume(Token::Semicolon, ParserErrorDescription::ExpectedToken(Token::Semicolon, "Expected ';' after value".into()))?;

        Ok(Stmt::Expression(value))
    }

    fn function(&mut self, kind: &str) -> ParserResult<Func> {
        let name = self.consume_discriminant(::std::mem::discriminant(&Token::Identifier(String::new())), ParserErrorDescription::ExpectedIdentifier(format!("Expected {} name", kind)))?.clone();

        let mut parameters = Vec::new();

        self.consume(Token::LeftParen, ParserErrorDescription::ExpectedToken(Token::LeftParen, format!("Expected '(' after {} name", kind)))?;
        if !self.check(Token::RightParen) {
            let parameter = self.consume_discriminant(::std::mem::discriminant(&Token::Identifier(String::new())), ParserErrorDescription::ExpectedIdentifier("Expected parameter name".into()))?;
            parameters.push(parameter.clone());
            while self.try_consume(Token::Comma) {
                if parameters.len() >= 255 {
                    return Err(self.error(self.peek(), ParserErrorDescription::TooManyParameters));
                }

                let parameter = self.consume_discriminant(::std::mem::discriminant(&Token::Identifier(String::new())), ParserErrorDescription::ExpectedIdentifier("Expected parameter name".into()))?;
                parameters.push(parameter.clone());
            }
        }
        self.consume(Token::RightParen, ParserErrorDescription::ExpectedToken(Token::RightParen, "Expected ')' after parameters".into()))?;

        let body = match self.statement()? {
            Stmt::Block(stmts) => {
                stmts
            },
            stmt => vec![stmt]
        };

        Ok(Func::new(name, parameters, body))
    }

    // expressions
    pub fn expression(&mut self) -> ParserResult<Expr> {
        self.assignment()
    }

    fn assignment(&mut self) -> ParserResult<Expr> {
        let expr = self.or()?;

        if self.try_consume(Token::Equal) {
            let equals = self.previous().clone();
            let value = self.assignment()?;

            match expr {
                Expr::Var(target) => {
                    Ok(Expr::Assign(target, Box::new(value)))
                }
                _ => {
                    Err(self.error(&equals, ParserErrorDescription::InvalidAssignmentTarget))
                }
            }

        } else {
            Ok(expr)
        }
    }

    fn or(&mut self) -> ParserResult<Expr> {
        let mut expr = self.and()?;

        while self.try_consume(Token::Or) {
            let operator = self.previous().clone();
            let right = self.and()?;

            expr = Expr::Logical(Box::new(expr), operator, Box::new(right));
        }

        Ok(expr)
    }

    fn and(&mut self) -> ParserResult<Expr> {
        let mut expr = self.equality()?;

        while self.try_consume(Token::And) {
            let operator = self.previous().clone();
            let right = self.equality()?;

            expr = Expr::Logical(Box::new(expr), operator, Box::new(right));
        }

        Ok(expr)
    }

    fn equality(&mut self) -> ParserResult<Expr> {
        let mut expr = self.comparison()?;

        while self.try_consume_one_of(vec![Token::BangEqual, Token::EqualEqual]) {
            let operator = self.previous().clone();
            let right = self.comparison()?;

            expr = Expr::Binary(Box::new(expr), operator, Box::new(right));
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> ParserResult<Expr> {
        let mut expr = self.addition()?;

        while self.try_consume_one_of(vec![Token::Greater, Token::GreaterEqual, Token::Less, Token::LessEqual]) {
            let operator = self.previous().clone();
            let right = self.addition()?;

            expr = Expr::Binary(Box::new(expr), operator, Box::new(right));
        }

        Ok(expr)
    }

    fn addition(&mut self) -> ParserResult<Expr> {
        let mut expr = self.multiplication()?;

        while self.try_consume_one_of(vec![Token::Minus, Token::Plus]) {
            let operator = self.previous().clone();
            let right = self.multiplication()?;

            expr = Expr::Binary(Box::new(expr), operator, Box::new(right));
        }

        Ok(expr)
    }

    fn multiplication(&mut self) -> ParserResult<Expr> {
        let mut expr = self.unary()?;

        while self.try_consume_one_of(vec![Token::Slash, Token::Star]) {
            let operator = self.previous().clone();
            let right = self.unary()?;

            expr = Expr::Binary(Box::new(expr), operator, Box::new(right));
        }

        Ok(expr)
    }

    fn unary(&mut self) -> ParserResult<Expr> {
        if self.try_consume_one_of(vec![Token::Bang, Token::Minus]) {
            let operator = self.previous().clone();
            let right = self.unary()?;

            Ok(Expr::Unary(operator, Box::new(right)))
        } else {
            self.call()
        }
    }

    fn call(&mut self) -> ParserResult<Expr> {
        let mut expr = self.primary()?;

        while self.try_consume(Token::LeftParen) {
            expr = self.finish_call(expr)?;
        }

        Ok(expr)
    }

    fn finish_call(&mut self, callee: Expr) -> ParserResult<Expr> {
        // left paren is already consumed
        let mut arguments = Vec::new();

        if !self.check(Token::RightParen) {
            arguments.push(self.expression()?);
            while self.try_consume(Token::Comma) {
                if arguments.len() >= 255 {
                    return Err(self.error(self.peek(), ParserErrorDescription::TooManyArguments));
                }

                arguments.push(self.expression()?);
            }
        }

        let paren = self.consume(Token::RightParen, ParserErrorDescription::ExpectedToken(Token::RightParen, "Expected ')' after arguments".into()))?;

        Ok(Expr::Call(Box::new(callee), paren.clone(), arguments))
    }

    fn primary(&mut self) -> ParserResult<Expr> {
        let token = self.advance();

        match &token.token {
            Token::False => Ok(Expr::Boolean(token.clone(), false)),
            Token::True => Ok(Expr::Boolean(token.clone(), true)),
            Token::Nil => Ok(Expr::Nil),

            Token::Number(val) => Ok(Expr::Number(token.clone(), *val)),
            Token::String(val) => Ok(Expr::String(token.clone(), val.clone())),

            Token::Identifier(_) => Ok(Expr::Var(token.clone())),

            Token::LeftParen => {
                if self.is_at_end() {
                    return Err(self.error(self.peek(), ParserErrorDescription::ExpectedExpression));
                }

                let expr = self.expression()?;
                self.consume(Token::RightParen, ParserErrorDescription::ExpectedToken(Token::RightParen, "Expected ')' after expression".into()))?;

                Ok(Expr::Grouping(Box::new(expr)))
            }

            _ => Err(self.error(self.peek(), ParserErrorDescription::ExpectedExpression)),
        }
    }

    // movement
    fn try_consume(&mut self, token: Token) -> bool {
        self.try_consume_discriminant(::std::mem::discriminant(&token))
    }
    fn try_consume_discriminant(&mut self, token: Discriminant<Token>) -> bool {
        if self.check_discriminant(token) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn try_consume_one_of(&mut self, tokens: Vec<Token>) -> bool {
        for token in tokens {
            if self.try_consume(token) {
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

    fn consume(&mut self, expected: Token, error: ParserErrorDescription) -> ParserResult<&SourceToken> {
        self.consume_discriminant(::std::mem::discriminant(&expected), error)
    }
    fn consume_discriminant(&mut self, expected: Discriminant<Token>, error: ParserErrorDescription) -> ParserResult<&SourceToken> {
        if self.check_discriminant(expected) {
           Ok(self.advance())
        } else {
            Err(self.error(self.peek(), error))
        }
    }

    fn error(&self, token: &SourceToken, description: ParserErrorDescription) -> ParserError {
        ParserError {
            line: token.line,
            location: if token.token == Token::Eof { "at end".into() } else { format!("at '{}'", token.lexeme) },
            description,
        }
    }

    fn synchronize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            if self.previous().token == Token::Semicolon {
                return;
            }

            match self.peek().token {
                Token::Class | Token::Fun | Token::Var | Token::For | Token::If | Token::While | Token::Print | Token::Return => return,
                _ => { }
            }

            self.advance();
        }
    }

    // checks
    fn check(&self, token: Token) -> bool {
        self.check_discriminant(::std::mem::discriminant(&token))
    }
    fn check_discriminant(&self, token: Discriminant<Token>) -> bool {
        if self.is_at_end() {
            false
        } else {
            ::std::mem::discriminant(&self.peek().token) == token
        }
    }

    fn is_at_end(&self) -> bool {
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
    use crate::stmt::Func;

    fn parse_statement(tokens: Vec<Token>) -> ParserResult<Stmt> {
        let mut source_tokens: Vec<SourceToken> = tokens.into_iter()
            .map(tok_to_src)
            .collect();
        source_tokens.push(tok_to_src(Token::Eof));

        let mut parser = Parser::new(source_tokens);
        parser.declaration()
    }
    fn expect_parse_statement(tokens: Vec<Token>) -> Stmt {
        parse_statement(tokens).expect("Failed to parse statement")
    }

    fn parse_expression(tokens: Vec<Token>) -> ParserResult<Expr> {
        let mut source_tokens: Vec<SourceToken> = tokens.into_iter()
            .map(tok_to_src)
            .collect();
        source_tokens.push(tok_to_src(Token::Eof));

        let mut parser = Parser::new(source_tokens);
        parser.expression()
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
    fn test_fun_declaration() {
        assert_eq!(expect_parse_statement(vec![Token::Fun, ident("abc"), Token::LeftParen, Token::RightParen, Token::LeftBrace, Token::RightBrace]), Stmt::Function(Func::new(tok_to_src(ident("abc")), vec![], vec![])));
        assert_eq!(expect_parse_statement(vec![Token::Fun, ident("abc"), Token::LeftParen, ident("a"), Token::RightParen, Token::LeftBrace, Token::RightBrace]), Stmt::Function(Func::new(tok_to_src(ident("abc")), vec![tok_to_src(ident("a"))], vec![])));
        assert_eq!(expect_parse_statement(vec![Token::Fun, ident("abc"), Token::LeftParen, ident("a"), Token::Comma, ident("b"), Token::RightParen, Token::LeftBrace, Token::RightBrace]), Stmt::Function(Func::new(tok_to_src(ident("abc")), vec![tok_to_src(ident("a")), tok_to_src(ident("b"))], vec![])));
        assert_eq!(expect_parse_statement(vec![Token::Fun, ident("abc"), Token::LeftParen, Token::RightParen, Token::LeftBrace, Token::Print, Token::Number(1f64), Token::Semicolon, Token::RightBrace]), Stmt::Function(Func::new(tok_to_src(ident("abc")), vec![], vec![Stmt::Print(expr_num(1f64))])));
    }

    #[test]
    fn test_var_declaration() {
        assert_eq!(expect_parse_statement(vec![Token::Var, ident("abc"), Token::Semicolon]), Stmt::Var(tok_to_src(ident("abc")), None));
        assert_eq!(expect_parse_statement(vec![Token::Var, ident("abc"), Token::Equal, Token::Number(123f64), Token::Semicolon]), Stmt::Var(tok_to_src(ident("abc")), Some(expr_num(123f64))));
    }

    #[test]
    fn test_for() {
        let empty_for = vec![Token::For, Token::LeftParen, Token::Semicolon, Token::Semicolon, Token::RightParen, Token::Print, Token::Number(2f64), Token::Semicolon];
        let just_init_for = vec![Token::For, Token::LeftParen, Token::Var, ident("a"), Token::Semicolon, Token::Semicolon, Token::RightParen, Token::Print, Token::Number(2f64), Token::Semicolon];
        let just_cond_for = vec![Token::For, Token::LeftParen, Token::Semicolon, Token::False, Token::Semicolon, Token::RightParen, Token::Print, Token::Number(2f64), Token::Semicolon];
        let just_update_for = vec![Token::For, Token::LeftParen, Token::Semicolon, Token::Semicolon, ident("a"), Token::Equal, Token::False, Token::RightParen, Token::Print, Token::Number(2f64), Token::Semicolon];
        let all_for = vec![Token::For, Token::LeftParen, Token::Var, ident("a"), Token::Semicolon, Token::Bang, ident("a"), Token::Semicolon, ident("a"), Token::Equal, Token::False, Token::RightParen, Token::Print, Token::Number(2f64), Token::Semicolon];

        assert_eq!(expect_parse_statement(empty_for),
                   Stmt::While(expr_bool(true), Box::new(Stmt::Print(expr_num(2f64)))));
        assert_eq!(expect_parse_statement(just_init_for),
                   Stmt::Block(vec![
                       Stmt::Var(tok_to_src(ident("a")), None),
                       Stmt::While(expr_bool(true), Box::new(Stmt::Print(expr_num(2f64)))),
                   ]));
        assert_eq!(expect_parse_statement(just_cond_for),
                   Stmt::While(expr_bool(false), Box::new(Stmt::Print(expr_num(2f64)))));
        assert_eq!(expect_parse_statement(just_update_for),
                   Stmt::While(expr_bool(true), Box::new(Stmt::Block(vec![
                       Stmt::Print(expr_num(2f64)),
                       Stmt::Expression(Expr::Assign(tok_to_src(ident("a")), Box::new(expr_bool(false))))
                   ]))));
        assert_eq!(expect_parse_statement(all_for),
                   Stmt::Block(vec![
                       Stmt::Var(tok_to_src(ident("a")), None),
                       Stmt::While(Expr::Unary(tok_to_src(Token::Bang), Box::new(Expr::Var(tok_to_src(ident("a"))))), Box::new(Stmt::Block(vec![
                           Stmt::Print(expr_num(2f64)),
                           Stmt::Expression(Expr::Assign(tok_to_src(ident("a")), Box::new(expr_bool(false))))
                       ]))),
                   ]));
    }

    #[test]
    fn test_if() {
        assert_eq!(expect_parse_statement(vec![Token::If, Token::LeftParen, Token::Number(1f64), Token::RightParen, Token::Print, Token::Number(2f64), Token::Semicolon]),
                   Stmt::If(expr_num(1f64), Box::new(Stmt::Print(expr_num(2f64))), None));
        assert_eq!(expect_parse_statement(vec![Token::If, Token::LeftParen, Token::Number(1f64), Token::RightParen, Token::Print, Token::Number(2f64), Token::Semicolon, Token::Else, Token::Print, Token::Number(3f64), Token::Semicolon]),
                   Stmt::If(expr_num(1f64), Box::new(Stmt::Print(expr_num(2f64))), Some(Box::new(Stmt::Print(expr_num(3f64))))));
    }

    #[test]
    fn test_print() {
        assert_eq!(expect_parse_statement(vec![Token::Print, Token::Number(123f64), Token::Semicolon]), Stmt::Print(expr_num(123f64)));
    }

    #[test]
    fn test_return() {
        assert_eq!(expect_parse_statement(vec![Token::Return, Token::Semicolon]), Stmt::Return(tok_to_src(Token::Return), None));
        assert_eq!(expect_parse_statement(vec![Token::Return, Token::Number(123f64), Token::Semicolon]), Stmt::Return(tok_to_src(Token::Return), Some(expr_num(123f64))));
    }

    #[test]
    fn test_while() {
        assert_eq!(expect_parse_statement(vec![Token::While, Token::LeftParen, Token::Number(123f64), Token::RightParen, Token::Print, Token::Number(456f64), Token::Semicolon]), Stmt::While(expr_num(123f64), Box::new(Stmt::Print(expr_num(456f64)))));
    }

    #[test]
    fn test_expression_statement() {
        assert_eq!(expect_parse_statement(vec![Token::Number(123f64), Token::Semicolon]), Stmt::Expression(expr_num(123f64)));
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