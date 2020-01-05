use std::io::{ self, Write };
use rlox_scanner::{ Scanner, ScannerError };

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

fn run(source: &String) -> Result<(), ScannerError> {
    let scanner = Scanner::new(source);
    let tokens = scanner.tokens();

    for token in tokens {
        let token = token?;

        println!("{:?}", token);
    }

    Ok(())
}