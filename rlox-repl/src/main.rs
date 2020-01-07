use std::io::{ self, Write };
use rlox_scanner::{ Scanner, ScannerError, Token };
use rlox_parser::{ Parser, ParserError };
use rlox_interpreter::{ Interpreter, RuntimeError as InterpreterError };

#[derive(Debug)]
enum ReplError {
    Scanner(ScannerError),
    Parser(ParserError),
    Interpreter(InterpreterError)
}

fn main() {
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    let mut interpreter = Interpreter::new();

    loop {
        print!("lox> ");
        stdout.flush().unwrap();

        let mut buffer = String::new();
        stdin.read_line(&mut buffer).unwrap();

        match run(&mut interpreter, &buffer) {
            Err(e) => eprintln!("{:?}", e),
            _ => { }
        }
    }
}

fn run(interpreter: &mut Interpreter, source: &String) -> Result<(), ReplError> {
    let scanner = Scanner::new(source);
    let mut tokens = Vec::new();
    for result in scanner.tokens() {
        let token = result.map_err(ReplError::Scanner)?;

        match &token.token {
            Token::NewLine | Token::Whitespace | Token::Comment => { }

            _ => tokens.push(token),
        }
    }

    let mut parser = Parser::new(tokens);
    let statements = parser.parse();

    for result in statements {
        let statement = result.map_err(ReplError::Parser)?;

        interpreter.interpret(vec![statement]).map_err(ReplError::Interpreter)?;
    }

    Ok(())
}