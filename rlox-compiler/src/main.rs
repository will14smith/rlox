use std::io::Write;
use std::rc::Rc;
use rlox_scanner::{ Scanner, ScannerError, Token };
use rlox_parser::{Parser, ParserError, StmtParser};
use rlox_compiler::{Chunk, Compiler, CompilerError, VM, VMError, disassemble_chunk};

#[derive(Debug)]
enum ReplError {
    Scanner(ScannerError),
    Parser(ParserError),
    Compiler(CompilerError),
    VM(VMError),
}

fn main() {
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();

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
            Token::NewLine | Token::Whitespace | Token::Comment => { }

            _ => tokens.push(token),
        }
    }

    let mut parser = Parser::new(tokens);
    let mut parser = StmtParser::new(&mut parser);
    let statements = parser.parse();

    let mut chunk = Chunk::new();
    let mut compiler = Compiler::new(&mut chunk);
    for result in statements {
        let statement = result.map_err(ReplError::Parser)?;

        compiler.compile(vec![statement]).map_err(ReplError::Compiler)?;
    }

    disassemble_chunk(&mut std::io::stdout(), &chunk);

    let mut vm = VM::new(Rc::new(chunk));
    vm.run().map_err(ReplError::VM)?;

    Ok(())
}