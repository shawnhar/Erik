mod input;
mod tokens;
mod expr;
mod ops;

use std::env;
use input::InputSource;
use tokens::Tokenizer;


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

    let expression = expr::parse_expression(&mut tokenizer, false)?;
    
    println!("{:#?}", expression);

    Ok(())
}
