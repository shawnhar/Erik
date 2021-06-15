#[derive(Debug)]
pub enum Token<'a> {
    Text(&'a str)
}


pub struct Tokenizer<'a> {
    input: &'a str,
    pos: usize
}


impl Tokenizer<'_> {
    pub fn new(input: &str) -> Tokenizer {
        Tokenizer { input, pos: 0 }
    }
}


impl<'a> Iterator for Tokenizer<'a> {
    type Item = Token<'a>;

    
    fn next(&mut self) -> Option<Token<'a>> {
        if self.pos < self.input.len() {
            let token = &self.input[self.pos..self.pos+1];
            self.pos = self.pos + 1;
            Some(Token::Text(token))
        }
        else {
            None
        }
    }
}
