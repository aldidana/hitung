use std::io;
use std::io::Write;

mod lexer;
mod token;

use lexer::Lexer;

fn main() {
    loop {
        println!();
        print!("> ");

        io::stdout()
            .flush()
            .expect("Error when flush stdout.");

        let mut input = String::new();

        io::stdin()
            .read_line(&mut input)
            .expect("Could not read from standard input.");

        let lexer = Lexer::new(input.as_str());
        let result = lexer.lex();

        println!("Result {:?}", result);
    }
}

