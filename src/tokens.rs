use std::str;
use crate::ops;


// Before parsing, input strings are tokenized into a linear sequence of these enums.
#[derive(Debug)]
pub enum Token<'a> {
    Number(f64),
    Text(&'a str),
    Operator(ops::OperatorRef),
}


// Tokenizer iterates over input characters, and is itself iterable as a series of tokens.
pub struct Tokenizer<'a> {
    input_iterator: str::Chars<'a>,
    remainder: &'a str,
    peeked: Option<char>,
}


impl<'a> Iterator for Tokenizer<'a> {
    type Item = Result<Token<'a>, String>;


    fn next(&mut self) -> Option<Result<Token<'a>, String>> {
        // Skip whitespace.
        while matches!(self.peek(), Some(char) if char.is_whitespace()) {
            self.get();
        }

        match self.peek() {
            Some(char) => {
                // Numbers.
                if char.is_ascii_digit() || char == '.' {
                    return Some(self.read_number());
                }

                // Barewords.
                if char.is_alphabetic() || (char == '_') {
                    return Some(Ok(self.read_bareword()));
                }

                // Quoted strings.
                if char == '"' || char == '\'' {
                    return Some(Ok(self.read_quoted()));
                }

                // Could this be an operator?
                if let Some(operator) = self.read_operator() {
                    return Some(Ok(operator));
                }

                // Unknown single character.
                return Some(Ok(self.read_unknown_character()));
            }
            
            // End of the input stream.
            None => None
        }
    }
}


impl<'a> Tokenizer<'a> {
    // Wraps a tokenizer around the provided string reference.
    pub fn new(input: &str) -> Tokenizer {
        Tokenizer {
            input_iterator: input.chars(),
            remainder: &input,
            peeked: None,
        }
    }


    // Reads the next character, advancing the input position.
    fn get(&mut self) -> Option<char> {
        let result = match self.peeked {
            // Consume a previously peeked value.
            Some(char) => {
                self.peeked = None;
                Some(char)
            }

            // Read a new value.
            None => self.input_iterator.next()
        };
        
        self.remainder = self.input_iterator.as_str();
        
        result
    }


    // Peeks the next character, without advancing the input position.
    fn peek(&mut self) -> Option<char> {
        if self.peeked.is_none() {
            self.peeked = self.input_iterator.next();
        }
        
        self.peeked
    }


    // Reads a numeric constant.
    fn read_number(&mut self) -> Result<Token<'a>, String> {
        let start_slice = self.remainder;

        if let Some('0') = self.get() {
            match self.peek() {
                Some('b') => { self.get(); return self.read_integer(2); }
                Some('x') => { self.get(); return self.read_integer(16); }
                _ => {}
            }
        }

        self.read_decimal(start_slice)
    }


    // Reads a decimal floating point constant.
    fn read_decimal(&mut self, start_slice: &str) -> Result<Token<'a>, String> {
        loop {
            match self.peek() {
                // Accept numeric digits and period characters.
                Some(char) if char.is_ascii_digit() || char == '.' => {
                    self.get();
                }

                // Also accept exponent markers, optionally followed by a minus sign.
                Some(char) if char == 'e' => {
                    self.get();
                    
                    if let Some('-') = self.peek() {
                        self.next();
                    }
                }

                _ => break
            }
        }

        let slice = &start_slice[..start_slice.len() - self.remainder.len()];

        // The above logic will accept plenty of invalid strings, so this conversion can fail!
        match slice.parse() {
            Ok(value) => Ok(Token::Number(value)),
            Err(_) => Err(format!("Invalid numeric constant '{}'.", slice)),
        }
    }


    // Reads an integer constant using binary or hexadecimal number base.
    fn read_integer(&mut self, base: u32) -> Result<Token<'a>, String> {
        let mut value = 0u32;

        while let Some(char) = self.peek() {
            if let Some(digit) = char.to_digit(base) {
                self.get();

                match value.checked_mul(base) {
                    Some(new_value) => value = new_value,
                    None => return Err(format!("Base {} constant overflowed 32 bit range.", base)),
                }
                
                value |= digit;
            } else {
                break;
            }
        }

        Ok(Token::Number(value as f64))
    }


    // Reads a alphabetical bareword.
    fn read_bareword(&mut self) -> Token<'a> {
        let start_slice = self.remainder;

        while matches!(self.peek(), Some(char) if char.is_alphanumeric() || char == '_') {
            self.get();
        }

        Token::Text(&start_slice[..start_slice.len() - self.remainder.len()])
    }


    // Reads a quoted string.
    fn read_quoted(&mut self) -> Token<'a> {
        let quote = self.get().unwrap();
        let start_slice = self.remainder;
        let mut end_slice = start_slice;

        loop {
            match self.get() {
                Some(char) if char == quote => break,
                Some(_) => end_slice = self.remainder,
                None => break,
            }
        }

        Token::Text(&start_slice[..start_slice.len() - end_slice.len()])
    }


    // Reads a single character.
    fn read_unknown_character(&mut self) -> Token<'a> {
        let start_slice = self.remainder;
        self.get();
        Token::Text(&start_slice[..start_slice.len() - self.remainder.len()])
    }


    // Attempts to match against the set of known operators.
    fn read_operator(&mut self) -> Option<Token<'a>> {
        let start_slice = self.remainder;

        fn could_be_operator(opname: &str) -> bool {
            ops::OPERATORS.iter().any(|op| op.name.starts_with(opname))
        }

        while could_be_operator(&start_slice[..start_slice.len() - self.input_iterator.as_str().len()]) {
            self.get();

            if self.peek().is_none() {
                break;
            }
        }

        let opname = &start_slice[..start_slice.len() - self.remainder.len()];

        ops::find_operator(opname).map(|operator| Token::Operator(operator))
    }
}


#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn get_and_peek() {
        let mut t = Tokenizer::new("abc");
        
        assert_eq!(t.remainder, "abc");
        
        assert_eq!(t.get().unwrap(), 'a');
        assert_eq!(t.remainder, "bc");

        assert_eq!(t.peek().unwrap(), 'b');
        assert_eq!(t.remainder, "bc");

        assert_eq!(t.peek().unwrap(), 'b');
        assert_eq!(t.remainder, "bc");

        assert_eq!(t.get().unwrap(), 'b');
        assert_eq!(t.remainder, "c");

        assert_eq!(t.get().unwrap(), 'c');
        assert_eq!(t.remainder, "");

        assert!(t.get() == None);
        assert!(t.peek() == None);
    }


    #[test]
    fn whitespace_and_barewords() {
        let mut t = Tokenizer::new("   hello  t  # [@ ");
        
        assert!(matches!(t.next().unwrap(), Ok(Token::Text("hello"))));
        assert!(matches!(t.next().unwrap(), Ok(Token::Text("t"))));
        assert!(matches!(t.next().unwrap(), Ok(Token::Text("#"))));
        assert!(matches!(t.next().unwrap(), Ok(Token::Text("["))));
        assert!(matches!(t.next().unwrap(), Ok(Token::Text("@"))));

        assert!(t.next().is_none());
    }


    #[test]
    fn quoted_strings() {
        let mut t = Tokenizer::new("   ' a b '  \"what's up\"  'unclosed ");
        
        assert!(matches!(t.next().unwrap(), Ok(Token::Text(" a b "))));
        assert!(matches!(t.next().unwrap(), Ok(Token::Text("what's up"))));
        assert!(matches!(t.next().unwrap(), Ok(Token::Text("unclosed "))));

        assert!(t.next().is_none());
    }


    fn expect_number(value: Option<Result<Token, String>>, expected: f64) {
        match value.unwrap().unwrap() {
            Token::Number(value) => assert_eq!(value, expected),
            _ => assert!(false)
        }
    }  
      

    #[test]
    fn floats() {
        let mut t = Tokenizer::new("1 100 0.5 3.14 .6 007 10e4 10e-3 1.5e2 0.5x -10 3ee2 3..14");

        expect_number(t.next(), 1.0);
        expect_number(t.next(), 100.0);
        expect_number(t.next(), 0.5);
        expect_number(t.next(), 3.14);
        expect_number(t.next(), 0.6);
        expect_number(t.next(), 7.0);
        expect_number(t.next(), 100000.0);
        expect_number(t.next(), 0.01);
        expect_number(t.next(), 150.0);

        expect_number(t.next(), 0.5);
        assert!(matches!(t.next().unwrap(), Ok(Token::Text("x"))));

        assert!(matches!(t.next().unwrap(), Ok(Token::Operator(_))));
        expect_number(t.next(), 10.0);

        assert_eq!(t.next().unwrap().unwrap_err(), "Invalid numeric constant '3ee2'.");
        assert_eq!(t.next().unwrap().unwrap_err(), "Invalid numeric constant '3..14'.");

        assert!(t.next().is_none());
    }


    #[test]
    fn hexadecimal() {
        let mut t = Tokenizer::new("0x 0x0 0x1 0xDeadBeef 0x12345678 0x9ABCDEF 0xffffffff 0xfeedme 0x100000000");

        expect_number(t.next(), 0.0);
        expect_number(t.next(), 0.0);
        expect_number(t.next(), 1.0);
        expect_number(t.next(), 0xDEADBEEFu32 as f64);
        expect_number(t.next(), 0x12345678 as f64);
        expect_number(t.next(), 0x9ABCDEF as f64);
        expect_number(t.next(), 0xFFFFFFFFu32 as f64);

        expect_number(t.next(), 0xFEED as f64);
        assert!(matches!(t.next().unwrap(), Ok(Token::Text("me"))));

        assert_eq!(t.next().unwrap().unwrap_err(), "Base 16 constant overflowed 32 bit range.");

        assert!(t.next().is_none());
    }


    #[test]
    fn binary() {
        let mut t = Tokenizer::new("0b 0b0 0b1 0b01101100 0b11111111111111111111111111111111 0b102 0b100000000000000000000000000000000");

        expect_number(t.next(), 0.0);
        expect_number(t.next(), 0.0);
        expect_number(t.next(), 1.0);
        expect_number(t.next(), 0x6Cu64 as f64);
        expect_number(t.next(), 0xFFFFFFFFu32 as f64);

        expect_number(t.next(), 2.0);
        assert!(matches!(t.next().unwrap(), Ok(Token::Number(_))));

        assert_eq!(t.next().unwrap().unwrap_err(), "Base 2 constant overflowed 32 bit range.");

        assert!(t.next().is_none());
    }


    #[test]
    fn operators() {
        let mut t = Tokenizer::new("x<y<=z!=");

        fn expect_operator(value: Option<Result<Token, String>>, expected: &str) {
            match value.unwrap().unwrap() {
                Token::Operator(value) => assert_eq!(value.name, expected),
                _ => assert!(false)
            }
        }  

        assert!(matches!(t.next().unwrap(), Ok(Token::Text("x"))));
        expect_operator(t.next(), "<");
        assert!(matches!(t.next().unwrap(), Ok(Token::Text("y"))));
        expect_operator(t.next(), "<=");
        assert!(matches!(t.next().unwrap(), Ok(Token::Text("z"))));
        expect_operator(t.next(), "!=");

        assert!(t.next().is_none());
    }
}
