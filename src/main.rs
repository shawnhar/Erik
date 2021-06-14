use std::env;

mod input;


fn main() {
    let args = env::args().skip(1).collect();

    let input = input::InputSource::new(args);

    for line in input {
        println!("{:?}", line);
    }
}
