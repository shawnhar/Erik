use std::env;
use std::fs;


pub struct InputSource {
    source_text: Vec<String>
}


impl InputSource {
    pub fn new() -> InputSource {
        let source_text = match Self::read_arg_file() {
            Some(arg_file_contents) => {
                // Source expression was read from an argument file.
                arg_file_contents.lines().map(String::from).collect()
            },

            None => {
                // Commandline arguments provide the source expression.
                env::args().skip(1).collect()
            },
        };

        InputSource { source_text: source_text }
    }


    // If there is only one commandline argument, try to read that as an argument file.
    fn read_arg_file() -> Option<String> {
        match env::args().nth(1) {
            Some(filename) if env::args().count() == 2 => fs::read_to_string(filename).ok(),
            _ => None
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
