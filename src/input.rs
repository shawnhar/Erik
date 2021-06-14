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


impl IntoIterator for InputSource {
    type Item = String;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.source_text.into_iter()
    }
}
