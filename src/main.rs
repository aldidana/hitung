use std::io;
use std::io::Write;

use inkwell::context::Context;

mod expression;
mod jit;
mod lexer;
mod parser;
mod token;

use jit::Compiler;

fn main() {
    let mut debug = false;

    for arg in std::env::args() {
        match arg.as_str() {
            "debug" => debug = true,
            _ => (),
        }
    }

    let context = Context::create();
    let mut compiler = Compiler::new(&context, debug);

    loop {
        // repl
        println!();
        print!("> ");

        io::stdout().flush().expect("Error when flush stdout.");

        let mut input = String::new();

        io::stdin()
            .read_line(&mut input)
            .expect("Could not read from standard input.");

        match compiler.compile_source(input.as_str()) {
            Ok(result) => println!("{}", result),
            Err(err) => {
                eprintln!("Error {:?}", err);
                break
            },
        }
    }
}
