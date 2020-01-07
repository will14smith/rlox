use rlox_scanner::{ Scanner, ScannerError, Token };
use rlox_parser::{ Parser, ParserError };
use rlox_interpreter::{ Interpreter, RuntimeError as InterpreterError };

#[derive(Debug)]
enum RloxError {
    Scanner(ScannerError),
    Parser(ParserError),
    Interpreter(InterpreterError)
}


fn main() {
    let args: Vec<String> = std::env::args().collect();
    std::process::exit(match args.len() {
        2 => {
            match run_file(&args[1]) {
                Ok(_) => 0,
                Err(code) => code,
            }
        }

        _ => {
            eprintln!("Usage: rlox <script>");
            64
        },
    })
}

fn run_file(file_name: &String) -> Result<(), i32> {
    let source = std::fs::read_to_string(file_name)
        .map_err(|e| { eprintln!("Failed to read source file: {:?}", e); 65 })?;

    let mut errors = Vec::new();

    let scanner = Scanner::new(&source);
    let mut tokens = Vec::new();
    for result in scanner.tokens() {
        let result = result.map_err(RloxError::Scanner);

        match result {
            Ok(token) => {
                match
                    &token.token {
                    Token::NewLine | Token::Whitespace | Token::Comment => {}

                    _ => tokens.push(token),
                }
            },

            Err(e) => errors.push(e),
        }
    }

    let mut parser = Parser::new(tokens);
    let mut statements = Vec::new();
    for result in parser.parse() {
        let result = result.map_err(RloxError::Parser);

        match result {
            Ok(statement) => statements.push(statement),
            Err(e) => errors.push(e),
        }
    }

    if !errors.is_empty() {
        for error in errors {
            eprintln!("Error: {:?}", error);
        }

        return Err(66);
    }

    let mut interpreter = Interpreter::new();
    interpreter.interpret(statements).map_err(RloxError::Interpreter)
        .map_err(|e| { eprintln!("Runtime error: {:?}", e); 70 })?;

    Ok(())
}