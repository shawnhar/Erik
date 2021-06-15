use std::str;


#[derive(Debug)]
pub enum Token<'a> {
    Number(f64),
    Integer(u64),
    Text(&'a str)
}


pub struct Tokenizer<'a> {
    iterator: str::Chars<'a>,
    remainder: &'a str,
    peeked: Option<char>
}


impl<'a> Iterator for Tokenizer<'a> {
    type Item = Token<'a>;

    
    fn next(&mut self) -> Option<Token<'a>> {
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
                    return Some(self.read_bareword());
                }

                // Quoted strings.
                if char == '"' || char == '\'' {
                    return Some(self.read_quoted());
                }

                // TODO a special operator?
                //object op = ReadOperator();

                //if (op != null)
                  //      return op;

                // Unknown single character.
                return Some(self.read_unknown_character());
            }
            
            // End of the input stream.
            None => return None
        }
    }
}


impl<'a> Tokenizer<'a> {
    pub fn new(input: &str) -> Tokenizer {
        Tokenizer {
            iterator: input.chars(),
            remainder: &input,
            peeked: None
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
            None => self.iterator.next()
        };
        
        self.remainder = self.iterator.as_str();
        
        result
    }


    // Peeks the next character, without advancing the input position.
    fn peek(&mut self) -> Option<char> {
        if let None = self.peeked {
            self.peeked = self.iterator.next();
        }
        
        self.peeked
    }


    // Reads a numeric constant.
    fn read_number(&mut self) -> Token<'a> {
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
    fn read_decimal(&mut self, start_slice: &str) -> Token<'a> {
        loop {
            match self.peek() {
                // Always accept numeric digits and period characters.
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

        let slice = &start_slice[.. start_slice.len() - self.remainder.len()];

        // The above logic will accept plenty of invalid strings, so this conversion can fail!
        // TODO error handling
        let value = slice.parse().unwrap();

        Token::Number(value)
    }


    // Reads an integer constant using binary or hexadecimal number base.
    fn read_integer(&mut self, base: u32) -> Token<'a> {
        let mut value = 0u64;

        while let Some(char) = self.peek() {
            if let Some(digit) = char.to_digit(base) {
                value *= base as u64;
                value |= digit as u64;
                self.get();
            }
            else {
                break;
            }
        }

        Token::Integer(value)
    }


    // Reads a alphabetical bareword.
    fn read_bareword(&mut self) -> Token<'a> {
        let start_slice = self.remainder;

        while matches!(self.peek(), Some(char) if char.is_alphabetic() || char == '_') {
            self.get();
        }

        Token::Text(&start_slice[.. start_slice.len() - self.remainder.len()])
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
                None => break
            }
        }

        Token::Text(&start_slice[.. start_slice.len() - end_slice.len()])
    }


    // Reads a single character.
    fn read_unknown_character(&mut self) -> Token<'a> {
        let start_slice = self.remainder;
        self.get();
        Token::Text(&start_slice[.. start_slice.len() - self.remainder.len()])
    }


/*
        // check if this could be an operator
        bool CouldBeOperator(string s)
        {
                foreach (string t in mOperators.Keys) {
                        if (t.StartsWith(s))
                                return true;
                }

                return false;
        }



        // read an operator
        object ReadOperator()
        {
                string s = "";

                while (CouldBeOperator(s + Peek()))
                        s += Next();

                return mOperators[s];
        }
        */

}
