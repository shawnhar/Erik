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
        loop {
            match self.peek() {
                Some(char) if char.is_whitespace() => { self.get(); }
                _ => break
            }
        }

        match self.peek() {
            Some(char) => {
                // Numbers.
                if char.is_ascii_digit() || char == '.' {
                    self.get();
                    return Some(Token::Number(23.0));
                    //              return ReadNumber();
                }

                // Barewords.
                if char.is_alphabetic() || (char == '_') {
                    return Some(Token::Text(self.read_bareword()));
                }

                // Quoted strings.
                if char == '"' || char == '\'' {
                    return Some(Token::Text(self.read_quoted()));
                }

                // TODO a special operator?
                //object op = ReadOperator();

                //if (op != null)
                  //      return op;

                // Unknown single character.
                return Some(Token::Text(self.read_unknown_character()));
            },
            
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



/*
    // Reads a numeric constant.
    fn read_number() -> f64 {
                string s = "";

                if (Peek() == '0') {
                        s += Next();

                        switch (Peek()) {
                                case 'x': Next(); return ReadHex();     
                                case 'b': Next(); return ReadBinary();  
                        }
                }

                return ReadDecimal(s);
        }



        // read a decimal floating point constant
        double ReadDecimal(string s)
        {
                while (char.IsDigit(Peek()) || (Peek() == '.') || (Peek() == 'e')) {
                        if (Peek() == 'e') {
                                s += Next();

                                if (Peek() == '-')
                                        s += Next();
                        }
                        else
                                s += Next();
                }

                return double.Parse(s, CultureInfo.InvariantCulture);
        }



        // convert a hex character to a numeric value
        static int Hex(char c)
        {
                if (char.IsDigit(c))
                        return c - '0';

                c = char.ToUpper(c);

                if ((c >= 'A') && (c <= 'F'))
                        return 0xA + c - 'A';

                return -1;
        }



        // read a hexadecimal constant
        double ReadHex()
        {
                ulong val = 0;

                while (Hex(Peek()) >= 0) {
                        val <<= 4;
                        val |= (uint)Hex(Next());
                }

                return val;
        }



        // read a binary constant
        double ReadBinary()
        {
                ulong val = 0;

                while ((Peek() == '0') || (Peek() == '1')) {
                        val <<= 1;
                        val |= (uint)(Next() - '0');
                }

                return val;
        }
*/


    // Reads a alphabetical bareword.
    fn read_bareword(&mut self) -> &'a str {
        let start_slice = self.remainder;

        loop {
            match self.peek() {
                Some(char) if char.is_alphabetic() || char == '_' => { self.get(); }
                _ => break
            }
        }

        &start_slice[.. start_slice.len() - self.remainder.len()]
    }


    // Reads a quoted string.
    fn read_quoted(&mut self) -> &'a str {
        let quote = self.get().unwrap();
        let start_slice = self.remainder;
        let mut end_slice = self.remainder;

        loop {
            match self.get() {
                Some(char) if char == quote => break,
                Some(_) => end_slice = self.remainder,
                None => break
            }
        }

        &start_slice[.. start_slice.len() - end_slice.len()]
    }


    // Reads a single character as a slice reference.
    fn read_unknown_character(&mut self) -> &'a str {
        let start_slice = self.remainder;
        self.get();
        &start_slice[.. start_slice.len() - self.remainder.len()]
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
