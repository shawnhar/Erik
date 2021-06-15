use std::env;

mod input;
mod tokens;


fn main() {
    let args = env::args().skip(1).collect();

    let input = input::InputSource::new(args);

    for line in input {
        println!("{:?}", line);

        let tokenizer = tokens::Tokenizer::new(&line);
        
        for token in tokenizer {
            println!("{:?}", token);
        }
    }
}
