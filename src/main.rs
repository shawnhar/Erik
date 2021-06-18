mod expr;
mod input;
mod ops;
mod tokens;

#[macro_use]
extern crate lazy_static;

use std::collections::HashMap;
use std::env;
use input::InputSource;
use tokens::Tokenizer;


// Global context stores all state of the calculator.
pub struct Context {

    // User defined functions.
    functions: HashMap<String, expr::Function>,

    // Where to write output, which can be redirected by unit tests.
    // TODO output: &<'a> dyn std::io::Write,
}


impl Context {
    pub fn new() -> Context {
        Context {
            functions: HashMap::new(),
            // TODO output: std::io::stdout(),
        }
    }
}


fn main() {
    let mut context = Context::new();
    
    // Skip over the executable name.
    let args = env::args().skip(1).collect();

    let input = InputSource::new(args);

    for line in input {
        if let Err(message) = evaluate_line(&line, &mut context) {
            println!("{}", message);
        }
    }
}


fn evaluate_line(line: &str, context: &mut Context) -> Result<(), String> {
    let mut tokenizer = Tokenizer::new(&line).peekable();

    while tokenizer.peek().is_some() {
        let mut expression = expr::parse(&mut tokenizer, false)?;

        if let Some((function, function_name)) = expr::deconstruct_function_definition(&mut expression) {
            // Define a new function.
            context.functions.insert(function_name, function);
        }
        else {
            // Evaluate an expression.
            println!("{}", expr::evaluate(&expression, context)?);
        }
    }

    Ok(())
}
