use std::env;

mod input;
mod tokens;
mod expr;
mod ops;


fn main() {
    let args = env::args().skip(1).collect();

    let input = input::InputSource::new(args);

    for line in input {
        evaluate(&line);
    }
}


fn evaluate(line: &str) {
    let tokenizer = tokens::Tokenizer::new(&line);
    
    for token in tokenizer {
        println!("{:?}", token);
    }
}
