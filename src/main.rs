mod expr;
mod input;
mod ops;
mod tokens;

#[macro_use]
extern crate lazy_static;

use std::collections::HashMap;
use std::env;
use expr::Function;
use input::InputSource;
use tokens::Tokenizer;


// Global context stores all state of the calculator.
pub struct Context {

    // User defined functions.
    functions: HashMap<String, Function>,

    // Where to write output, which can be redirected by unit tests.
    // TODO output: &<'a> dyn std::io::Write,
}


fn main() {
    let context = Context {
        functions: HashMap::new(),
        // TODO output: std::io::stdout(),
    };
    
    // Skip over the executable name.
    let args = env::args().skip(1).collect();

    let input = InputSource::new(args);

    for line in input {
        if let Err(message) = evaluate_line(&line, &context) {
            println!("{}", message);
        }
    }
}


fn evaluate_line(line: &str, context: &Context) -> Result<(), String> {
    let mut tokenizer = Tokenizer::new(&line).peekable();

    let expression = expr::parse(&mut tokenizer, false)?;

    println!("{}", expression);
    println!("{}", expr::evaluate(&expression, context)?);

    Ok(())
}
