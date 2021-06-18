mod expr;
mod input;
mod ops;
mod tokens;

use std::env;
use input::InputSource;
use tokens::Tokenizer;

#[macro_use]
extern crate lazy_static;


fn main() {
    let args = env::args().skip(1).collect();

    let input = InputSource::new(args);

    for line in input {
        if let Err(message) = evaluate(&line) {
            println!("{}", message);
        }
    }
}


fn evaluate(line: &str) -> Result<(), String> {
    let mut tokenizer = Tokenizer::new(&line).peekable();

    let expression = expr::parse(&mut tokenizer, false)?;

    println!("{}", expression);
    println!("{}", expr::evaluate(&expression)?);

    Ok(())
}
