use std::fs;


pub struct InputSource {
    source_text: Vec<String>
}


impl InputSource {
    pub fn new(args: Vec<String>) -> InputSource {
        // Should we read an argument file, or use the commandline arguments directly?
        let source_text = match Self::read_arg_file(&args) {
            Some(arg_file_contents) => arg_file_contents,
            None => args
        };

        InputSource { source_text: source_text }
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
}


// We can iterate over the input source, which will yield a series of strings.
impl IntoIterator for InputSource {
    type Item = String;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.source_text.into_iter()
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_args() {
        let input = InputSource::new(vec![]);
        let mut iter = input.into_iter();

        assert!(iter.next() == None);
    }

    #[test]
    fn test_one_arg() {
        let input = InputSource::new(vec![ String::from("Hello") ]);
        let mut iter = input.into_iter();

        assert_eq!(iter.next().unwrap(), String::from("Hello"));
        assert!(iter.next() == None);
    }

    #[test]
    fn test_two_args() {
        let input = InputSource::new(vec![ String::from("Hello"), String::from("World") ]);
        let mut iter = input.into_iter();

        assert_eq!(iter.next().unwrap(), String::from("Hello"));
        assert_eq!(iter.next().unwrap(), String::from("World"));
        assert!(iter.next() == None);
    }

    #[test]
    fn test_one_arg_file_exists() {
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
    fn test_two_args_file_exists() {
        fs::write("args2.txt", "This\nis a\ntest").unwrap();

        let input = InputSource::new(vec![ String::from("args.txt"), String::from("another") ]);
        let mut iter = input.into_iter();

        assert_eq!(iter.next().unwrap(), String::from("args.txt"));
        assert_eq!(iter.next().unwrap(), String::from("another"));
        assert!(iter.next() == None);
        
        fs::remove_file("args2.txt").unwrap();
    }
}
