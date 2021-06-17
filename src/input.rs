use std::fs;
use std::io;
use std::io::Write;


pub struct InputSource {
    text: Option<Vec<String>>
}


impl InputSource {
    pub fn new(args: Vec<String>) -> InputSource {
        if args.len() == 0 {
            // Reading from an interactive console.
            InputSource { text: None }
        }
        else {
            // Should we read an argument file, or use the commandline arguments directly?
            let mut text = match read_arg_file(&args) {
                Some(arg_file_contents) => arg_file_contents,
                None => vec![ args.join(" ") ]
            };

            text.reverse();

            InputSource { text: Some(text) }
        }
    }
}


// If there is only one commandline argument, try to read that as an argument file.
fn read_arg_file(args: &[String]) -> Option<Vec<String>> {
    if args.len() == 1 {
        let filename = &args[0];
        
        match fs::read_to_string(filename) {
            Ok(file_contents) => Some(file_contents.lines()
                                                   .map(String::from)
                                                   .collect()),
            Err(_) => None
        }
    }
    else {
        None
    }
}


// Iterating over the input source yields a series of strings.
impl Iterator for InputSource {
    type Item = String;


    fn next(&mut self) -> Option<String> {
        match &mut self.text {
            Some(text) => {
                // Return text from commandline or argument file.
                text.pop()
            },
            
            None => {
                // Read text from the console.
                print!("\n> ");

                if io::stdout().flush().is_err() {
                    return None;
                }

                let mut line = String::new();

                match io::stdin().read_line(&mut line) {
                    Ok(_) if !line.trim().is_empty() => Some(line),
                    _ => None
                }
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn one_arg() {
        let input = InputSource::new(vec![ String::from("Hello") ]);
        let mut iter = input.into_iter();

        assert_eq!(iter.next().unwrap(), String::from("Hello"));
        assert!(iter.next() == None);
    }


    #[test]
    fn two_args() {
        let input = InputSource::new(vec![ String::from("Hello"), String::from("World") ]);
        let mut iter = input.into_iter();

        assert_eq!(iter.next().unwrap(), String::from("Hello World"));
        assert!(iter.next() == None);
    }


    #[test]
    fn one_arg_file_exists() {
        fs::write("args.txt", "This\nis a\ntest").unwrap();

        let input = InputSource::new(vec![ String::from("args.txt") ]);
        let mut iter = input.into_iter();

        assert_eq!(iter.next().unwrap(), String::from("This"));
        assert_eq!(iter.next().unwrap(), String::from("is a"));
        assert_eq!(iter.next().unwrap(), String::from("test"));
        assert!(iter.next() == None);
        
        fs::remove_file("args.txt").unwrap();
    }


    #[test]
    fn two_args_file_exists() {
        fs::write("args2.txt", "This\nis a\ntest").unwrap();

        let input = InputSource::new(vec![ String::from("args.txt"), String::from("another") ]);
        let mut iter = input.into_iter();

        assert_eq!(iter.next().unwrap(), String::from("args.txt another"));
        assert!(iter.next() == None);
        
        fs::remove_file("args2.txt").unwrap();
    }
}
