use rlox_scanner::Token;
use crate::parser::{ Parser, ParserErrorDescription, ParserResult };
use crate::expr_parser::ExprParser;
use crate::{ Expr, Func, Stmt };

pub struct StmtParser<'a> {
    parser: &'a mut Parser
}

impl<'a> StmtParser<'a> {
    pub fn new(parser: &'a mut Parser) -> StmtParser<'a> {
        StmtParser {
            parser
        }
    }

    pub fn parse(&mut self) -> Vec<ParserResult<Stmt>> {
        let mut statements = Vec::new();

        while !self.parser.is_at_end() {
            statements.push(self.declaration());
        }

        statements
    }

    // statements
    fn declaration(&mut self) -> ParserResult<Stmt> {
        let decl = if self.parser.try_consume(Token::Class) {
            self.class_declaration()
        } else if self.parser.try_consume(Token::Fun) {
            self.function("function").map(Stmt::Function)
        } else if self.parser.try_consume(Token::Var) {
            self.var_declaration()
        } else {
            self.statement()
        };

        match decl {
            Ok(stmt) => Ok(stmt),
            Err(err) => {
                self.synchronize();
                Err(err)
            },
        }
    }

    fn class_declaration(&mut self) -> ParserResult<Stmt> {
        // class keyword is already consumed
        let name = self.parser.consume_discriminant(::std::mem::discriminant(&Token::Identifier(String::new())), ParserErrorDescription::ExpectedIdentifier("Expected class name".into()))?;
        let name = name.clone();

        self.parser.consume(Token::LeftBrace, ParserErrorDescription::ExpectedToken(Token::LeftBrace, "Expected '{' before class body".into()))?;

        let mut functions = Vec::new();
        while !self.parser.check(Token::RightBrace) && !self.parser.is_at_end() {
            functions.push(self.function("method")?);
        }

        self.parser.consume(Token::RightBrace, ParserErrorDescription::ExpectedToken(Token::RightBrace, "Expected '}' after class body".into()))?;

        Ok(Stmt::Class(name, functions))
    }

    fn var_declaration(&mut self) -> ParserResult<Stmt> {
        // var keyword is already consumed
        let name = self.parser.consume_discriminant(::std::mem::discriminant(&Token::Identifier(String::new())), ParserErrorDescription::ExpectedIdentifier("Expected variable name".into()))?;
        let name = name.clone();

        let initializer = if self.parser.try_consume(Token::Equal) {
            Some(self.expression()?)
        } else {
            None
        };

        self.parser.consume(Token::Semicolon, ParserErrorDescription::ExpectedToken(Token::Semicolon, "Expected ';' after variable declaration".into()))?;

        Ok(Stmt::Var(name, initializer))
    }

    fn statement(&mut self) -> ParserResult<Stmt> {
        if self.parser.try_consume(Token::For) {
            self.for_statement()
        } else if self.parser.try_consume(Token::If) {
            self.if_statement()
        } else if self.parser.try_consume(Token::Print) {
            self.print_statement()
        } else if self.parser.try_consume(Token::Return) {
            self.return_statement()
        } else if self.parser.try_consume(Token::While) {
            self.while_statement()
        } else if self.parser.try_consume(Token::LeftBrace) {
            Ok(Stmt::Block(self.block()?))
        } else {
            self.expression_statement()
        }
    }

    fn for_statement(&mut self) -> ParserResult<Stmt> {
        // for keyword is already consumed
        self.parser.consume(Token::LeftParen, ParserErrorDescription::ExpectedToken(Token::LeftParen, "Expected '(' after 'for'".into()))?;

        let initializer = if self.parser.try_consume(Token::Semicolon) {
            None
        } else if self.parser.try_consume(Token::Var) {
            Some(self.var_declaration()?)
        } else {
            Some(self.expression_statement()?)
        };

        let condition = if self.parser.check(Token::Semicolon) {
            Expr::Boolean(self.parser.previous().clone(), true)
        } else {
            self.expression()?
        };
        self.parser.consume(Token::Semicolon, ParserErrorDescription::ExpectedToken(Token::Semicolon, "Expected ';' after for condition".into()))?;

        let update = if self.parser.check(Token::RightParen) {
            None
        } else {
            Some(self.expression()?)
        };
        self.parser.consume(Token::RightParen, ParserErrorDescription::ExpectedToken(Token::RightParen, "Expected ')' after for update".into()))?;

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
        self.parser.consume(Token::LeftParen, ParserErrorDescription::ExpectedToken(Token::LeftParen, "Expected '(' after 'if'".into()))?;
        let condition = self.expression()?;
        self.parser.consume(Token::RightParen, ParserErrorDescription::ExpectedToken(Token::RightParen, "Expected ')' after if condition".into()))?;

        let then_branch = Box::new(self.statement()?);
        let else_branch = if self.parser.try_consume(Token::Else) {
            Some(Box::new(self.statement()?))
        } else {
            None
        };

        Ok(Stmt::If(condition, then_branch, else_branch))
    }

    fn print_statement(&mut self) -> ParserResult<Stmt> {
        // print keyword is already consumed
        let value = self.expression()?;

        self.parser.consume(Token::Semicolon, ParserErrorDescription::ExpectedToken(Token::Semicolon, "Expected ';' after value".into()))?;

        Ok(Stmt::Print(value))
    }

    fn return_statement(&mut self) -> ParserResult<Stmt> {
        let token = self.parser.previous().clone();

        let value = if self.parser.check(Token::Semicolon) {
            None
        } else {
            Some(self.expression()?)
        };

        self.parser.consume(Token::Semicolon, ParserErrorDescription::ExpectedToken(Token::Semicolon, "Expected ';' after return value".into()))?;

        Ok(Stmt::Return(token, value))
    }

    fn while_statement(&mut self) -> ParserResult<Stmt> {
        // while keyword is already consumed
        self.parser.consume(Token::LeftParen, ParserErrorDescription::ExpectedToken(Token::LeftParen, "Expected '(' after 'if'".into()))?;
        let condition = self.expression()?;
        self.parser.consume(Token::RightParen, ParserErrorDescription::ExpectedToken(Token::RightParen, "Expected ')' after if condition".into()))?;

        let body = Box::new(self.statement()?);

        Ok(Stmt::While(condition, body))
    }

    fn block(&mut self) -> ParserResult<Vec<Stmt>> {
        // left brace is already consumed
        let mut statements = Vec::new();

        while !self.parser.check(Token::RightBrace) && !self.parser.is_at_end() {
            statements.push(self.declaration()?);
        }

        self.parser.consume(Token::RightBrace, ParserErrorDescription::ExpectedToken(Token::RightBrace, "Expected '}' after block".into()))?;

        Ok(statements)
    }

    fn expression_statement(&mut self) -> ParserResult<Stmt> {
        let value = self.expression()?;

        self.parser.consume(Token::Semicolon, ParserErrorDescription::ExpectedToken(Token::Semicolon, "Expected ';' after value".into()))?;

        Ok(Stmt::Expression(value))
    }

    fn function(&mut self, kind: &str) -> ParserResult<Func> {
        let name = self.parser.consume_discriminant(::std::mem::discriminant(&Token::Identifier(String::new())), ParserErrorDescription::ExpectedIdentifier(format!("Expected {} name", kind)))?.clone();

        let mut parameters = Vec::new();

        self.parser.consume(Token::LeftParen, ParserErrorDescription::ExpectedToken(Token::LeftParen, format!("Expected '(' after {} name", kind)))?;
        if !self.parser.check(Token::RightParen) {
            let parameter = self.parser.consume_discriminant(::std::mem::discriminant(&Token::Identifier(String::new())), ParserErrorDescription::ExpectedIdentifier("Expected parameter name".into()))?;
            parameters.push(parameter.clone());
            while self.parser.try_consume(Token::Comma) {
                if parameters.len() >= 255 {
                    return Err(self.parser.error(self.parser.peek(), ParserErrorDescription::TooManyParameters));
                }

                let parameter = self.parser.consume_discriminant(::std::mem::discriminant(&Token::Identifier(String::new())), ParserErrorDescription::ExpectedIdentifier("Expected parameter name".into()))?;
                parameters.push(parameter.clone());
            }
        }
        self.parser.consume(Token::RightParen, ParserErrorDescription::ExpectedToken(Token::RightParen, "Expected ')' after parameters".into()))?;

        let body = match self.statement()? {
            Stmt::Block(stmts) => {
                stmts
            },
            stmt => vec![stmt]
        };

        Ok(Func::new(name, parameters, body))
    }

    fn expression(&mut self) -> ParserResult<Expr> {
        let mut expr_parser = ExprParser::new(self.parser);
        expr_parser.parse()
    }

    fn synchronize(&mut self) {
        self.parser.advance();

        while !self.parser.is_at_end() {
            if self.parser.previous().token == Token::Semicolon {
                return;
            }

            match self.parser.peek().token {
                Token::Class | Token::Fun | Token::Var | Token::For | Token::If | Token::While | Token::Print | Token::Return => return,
                _ => { }
            }

            self.parser.advance();
        }
    }
}

#[cfg(test)]
mod tests {
    use rlox_scanner::SourceToken;
    use crate::Expr;
    use super::*;

    fn parse_statement(tokens: Vec<Token>) -> ParserResult<Stmt> {
        let mut source_tokens: Vec<SourceToken> = tokens.into_iter()
            .map(tok_to_src)
            .collect();
        source_tokens.push(tok_to_src(Token::Eof));

        let mut parser = Parser::new(source_tokens);
        let mut stmt_parser = StmtParser::new(&mut parser);

        stmt_parser.declaration()
    }
    fn expect_parse_statement(tokens: Vec<Token>) -> Stmt {
        parse_statement(tokens).expect("Failed to parse statement")
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

        let blank_true = Expr::Boolean(tok_to_src(Token::Semicolon), true);

        assert_eq!(expect_parse_statement(empty_for),
                   Stmt::While(blank_true.clone(), Box::new(Stmt::Print(expr_num(2f64)))));
        assert_eq!(expect_parse_statement(just_init_for),
                   Stmt::Block(vec![
                       Stmt::Var(tok_to_src(ident("a")), None),
                       Stmt::While(blank_true.clone(), Box::new(Stmt::Print(expr_num(2f64)))),
                   ]));
        assert_eq!(expect_parse_statement(just_cond_for),
                   Stmt::While(expr_bool(false), Box::new(Stmt::Print(expr_num(2f64)))));
        assert_eq!(expect_parse_statement(just_update_for),
                   Stmt::While(blank_true.clone(), Box::new(Stmt::Block(vec![
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
}