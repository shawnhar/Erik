mod expr;
mod input;
mod ops;
mod tokens;

#[macro_use]
extern crate lazy_static;

use std::collections::HashMap;
use std::env;
use std::iter::Peekable;
use input::InputSource;
use tokens::{Token, Tokenizer};


// Global context stores all state of the calculator.
pub struct Context {

    // User defined functions.
    functions: HashMap<String, expr::Function>,
}


impl Context {
    pub fn new() -> Context {
        Context {
            functions: HashMap::new()
        }
    }
}


fn main() {
    let mut context = Context::new();
    
    // Skip over the executable name.
    let args = env::args().skip(1).collect();

    let input = InputSource::new(args);

    for line in input {
        match evaluate_line(&line, &mut context) {
            Ok(true)     => {},
            Ok(false)    => break,
            Err(message) => println!("{}", message)
        }
    }
}


fn evaluate_line(line: &str, context: &mut Context) -> Result<bool, String> {
    let mut tokenizer = Tokenizer::new(&line).peekable();

    // Is this a special command?
    if let Some(result) = dispatch_command(&mut tokenizer, context) {
        return Ok(result);
    }
    
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

    Ok(true)
}


fn dispatch_command(tokenizer: &mut Peekable<Tokenizer>, context: &mut Context) -> Option<bool> {
    // Check if the next input token is in the COMMANDS table, and dispatch through that if found.
    if let Some(Ok(Token::Text(command))) = tokenizer.peek() {
        if let Some(command) = COMMANDS.get(command) {
            return Some(command(context))
        }
    }

    None
}


// Special commands return a bool indicating whether to keep going.
type Command = Box<fn(&Context) -> bool>;


lazy_static! {
    static ref COMMANDS: HashMap<&'static str, Command> = [
        ( "q",    Command::new(quit_command) ),
        ( "quit", Command::new(quit_command) ),
        ( "exit", Command::new(quit_command) ),
        ( "ls",   Command::new(ls_command)   ),
    ].iter().cloned().collect();
}


fn quit_command(_: &Context) -> bool {
    false
}


fn ls_command(context: &Context) -> bool {
    use itertools::Itertools;

    let sorted_functions = context.functions.iter().sorted_by_key(|f| f.0);
    
    for (name, expr::Function{ expression, args }) in sorted_functions {
        let args = if args.is_empty() {
            String::from("")
        }
        else {
            String::from("(") + &args.join(",") + ")"
        };
        
        println!("{}{} = {}", name, args, expression);
    }
    
    true
}
