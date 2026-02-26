use memmap2::Mmap;
use std::fs::File;
use std::io::{self, BufRead, Write};
use std::path::Path;
use std::time::Instant;

use regula_project::{Lexer, Parser, Token, TokenType};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let filtered_args: Vec<String> = args
        .iter()
        .filter(|s| s.as_str() != "-v")
        .cloned()
        .collect();
    let verbose = args.contains(&"-v".to_string());

    match filtered_args.len() {
        1 => run_repl(verbose),
        2 => {
            let filename = &filtered_args[1];
            run_file(filename, verbose);
        }
        _ => {
            eprintln!("Usage: regula [script]");
            std::process::exit(1);
        }
    }
}

fn run_repl(verbose: bool) {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    println!("Regula REPL (type 'exit' to exit)");

    loop {
        print!(">> ");
        stdout.flush().unwrap();
        let mut line = String::new();
        if stdin.lock().read_line(&mut line).is_err() {
            println!();
            break;
        }
        let line = line.trim();
        if line == "exit" {
            break;
        }
        if line.is_empty() {
            continue;
        }

        execute(line, verbose);
    }
    println!("Goodbye!")
}

fn run_file(filename: &str, verbose: bool) {
    let path = Path::new(filename);
    let file = match File::open(path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Failed to open file '{}': {}", filename, e);
            std::process::exit(1);
        }
    };

    let mmap = match unsafe { Mmap::map(&file) } {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Failed to mmap file '{}': {}", filename, e);
            std::process::exit(1);
        }
    };

    let source = match std::str::from_utf8(&mmap) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("File '{}' is not valid UTF-8: {}", filename, e);
            std::process::exit(1);
        }
    };

    execute(source, verbose);
}

fn execute(source: &str, verbose: bool) {
    let start = Instant::now();

    let source_bytes = source.as_bytes();
    let mut lexer = Lexer::new(String::from("<stdin>"), source_bytes);
    let mut tokens: Vec<Token> = vec![];
    loop {
        let token = lexer.next_token();
        match token {
            Ok(tok) => {
                if tok.token_type == TokenType::Eof {
                    tokens.push(tok);
                    break;
                }

                tokens.push(tok);
            }
            Err(e) => {
                eprintln!("{}", e);
                return;
            }
        }
    }

    let mut parser = Parser::new(&tokens, source_bytes, String::from("<stdin>"));
    let ast = parser.parse();
    match ast {
        Ok(ast) => {
            if !verbose {
                println!("{:#?}", ast)
            }
        }
        Err(e) => {
            eprintln!("{}", e);
            return;
        }
    }

    println!("Elapsed: {:?}", start.elapsed());
}
