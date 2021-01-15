use std::io;
use std::io::Write;

mod expression;
mod lexer;
mod parser;
mod token;

use lexer::Lexer;
use parser::Parser;

fn main() {
    loop {
        println!();
        print!("> ");

        io::stdout().flush().expect("Error when flush stdout.");

        let mut input = String::new();

        io::stdin()
            .read_line(&mut input)
            .expect("Could not read from standard input.");

        let lexer = Lexer::new(input.as_str());
        let result = lexer.lex();

        let mut parser = Parser::new(result);
        match parser.expr(0) {
            Ok(expression) => println!("Result {:?}", expression),
            Err(err) => eprintln!("Error {:?}", err),
        }
    }
}
