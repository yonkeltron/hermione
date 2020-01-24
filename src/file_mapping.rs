use colored::*;
use serde::{Deserialize, Serialize};

#[cfg(test)]
use quickcheck_macros::quickcheck;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct FileMapping {
    i: String,
    o: String,
}

impl FileMapping {
    pub fn new(i: &str, o: &str) -> FileMapping {
        FileMapping {
            i: String::from(i),
            o: String::from(o),
        }
    }

    pub fn display_line(self) -> String {
        format!("{} -> {}", self.i.green(), self.o.green())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constructor() {
        let i = "panda";
        let o = "bamboo";

        let expected = FileMapping::new(i, o);
        let actual = FileMapping {
            i: String::from(i),
            o: String::from(o),
        };

        assert_eq!(expected, actual);
    }

    #[quickcheck]
    fn test_display_line(a: String, b: String) -> bool {
        let file_mapping = FileMapping::new(&a, &b);
        let display_line = file_mapping.display_line();

        display_line.contains(&a) && display_line.contains(&b) && display_line.contains(" -> ")
    }
}
