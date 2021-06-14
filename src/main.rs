mod input;


fn main() {
    let input = input::InputSource::new();

    for line in input {
        println!("{:?}", line);
    }
}
