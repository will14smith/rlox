use std::io::{ self, Write };
use rlox_scanner::{ Scanner, ScannerError, Token };
use rlox_parser::{ Parser, ParserError };
use rlox_interpreter::{ RuntimeError as InterpreterError };

#[derive(Debug)]
enum ReplError {
    Scanner(ScannerError),
    Parser(ParserError),
    Interpreter(InterpreterError)
}

fn main() {
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    loop {
        print!("lox> ");
        stdout.flush().unwrap();

        let mut buffer = String::new();
        stdin.read_line(&mut buffer).unwrap();

        match run(&buffer) {
            Err(e) => eprintln!("{:?}", e),
            _ => { }
        }
    }
}

fn run(source: &String) -> Result<(), ReplError> {
    let scanner = Scanner::new(source);
    let mut tokens = Vec::new();
    for result in scanner.tokens() {
        let token = result.map_err(ReplError::Scanner)?;

        match &token.token {
            Token::NewLine | Token::Whitespace => { }

            _ => tokens.push(token),
        }
    }

    let mut parser = Parser::new(tokens);
    let expr = parser.parse().map_err(ReplError::Parser)?;

    let value = rlox_interpreter::evaluate_expression(&expr).map_err(ReplError::Interpreter)?;
    println!("{}", value);

    Ok(())
}