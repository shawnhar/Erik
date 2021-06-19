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
    
    // What number base(s) to display output in.
    bases: Vec<u32>,
}


impl Context {
    pub fn new() -> Context {
        Context {
            functions: HashMap::new(),
            bases: vec![ 10 ],
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
            print_number(expr::evaluate(&expression, context)?, &context.bases);
        }
    }

    Ok(true)
}


fn dispatch_command(tokenizer: &mut Peekable<Tokenizer>, context: &mut Context) -> Option<bool> {
    // Check if the next input token is in the COMMANDS table, and dispatch through that if found.
    if let Some(Ok(Token::Text(command))) = tokenizer.peek() {
        if let Some(command) = COMMANDS.get(command) {
            tokenizer.next();
            
            return Some(command(tokenizer, context))
        }
    }

    None
}


// Special commands return a bool indicating whether to keep going.
type Command = Box<fn(&mut Peekable<Tokenizer>, &mut Context) -> bool>;


lazy_static! {
    static ref COMMANDS: HashMap<&'static str, Command> = [
        ( "q",    Command::new(quit_command) ),
        ( "quit", Command::new(quit_command) ),
        ( "exit", Command::new(quit_command) ),
        ( "ls",   Command::new(ls_command)   ),
        ( "help", Command::new(help_command) ),
        ( "base", Command::new(base_command) ),
    ].iter().cloned().collect();
}


fn quit_command(_: &mut Peekable<Tokenizer>, _: &mut Context) -> bool {
    false
}


fn ls_command(_: &mut Peekable<Tokenizer>, context: &mut Context) -> bool {
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


fn help_command(_: &mut Peekable<Tokenizer>, _: &mut Context) -> bool {
    print_help("Operators", ops::OPERATORS.iter().map(|op| op.name).collect());
    print_help("Functions", ops::FUNCTIONS.iter().map(|op| op.name).collect());
    print_help("Commands",  COMMANDS      .iter().map(|cmd| *cmd.0).collect());

    true
}


fn print_help(title: &str, mut items: Vec<&str>) {
    println!();
    println!("{}:", title);

    items.sort();

    let item_width = items.iter().fold(0, |a, i| std::cmp::max(a, i.len())) + 2;

    for line in items.chunks(60 / item_width) {
        print!("    ");

        for item in line {        
            print!("{:w$}", item, w = item_width);
        }
        
        println!();
    }
}


fn base_command(tokenizer: &mut Peekable<Tokenizer>, context: &mut Context) -> bool {
    let mut new_bases = vec![];

    for token in tokenizer {
        match token {
            Ok(Token::Number(base)) if base >= 2.0 && base <= 36.0 => new_bases.push(base as u32),
            _ => { println!("Usage: base <list of number bases between 2 and 36>"); return true; }
        }
    }

    if !new_bases.is_empty() {
        context.bases = new_bases;
    }

    println!("Using base {}", context.bases.iter()
                                           .map(|b| b.to_string())
                                           .collect::<Vec<String>>()
                                           .join(" "));

    true
}


fn print_number(value: f64, bases: &Vec<u32>) {
    for (i, base) in bases.iter().enumerate() {
        if i > 0 {
            print!("  ");
        }
        
        match base {
            10 => print!("{}",   value),
            16 => print!("0x{}", format_integer(value, *base)),
            _  => print!("{}",   format_integer(value, *base)),
        }
    }
    
    println!();
}


fn format_integer(value: f64, base: u32) -> String {
    let value = value as i64 as u32;
    let base = base as u64;
    let mut p = base;
    let mut i = 1;

    let mut result = String::new();

    while p <= value as u64 {
        p *= base;
        i = i + 1;
    }

    while i > 0 {
        i = i - 1;
        p = p / base;
        
        result.push(std::char::from_digit(((value as u64 / p) % base) as u32, base as u32).unwrap());
        
        if base == 2 && i > 0 && i%4 == 0 {
            result.push('_');
        }
    }

    result
}
