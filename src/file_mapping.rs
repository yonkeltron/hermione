use anyhow::{anyhow, Result};
use colored::*;

#[cfg(test)]
use quickcheck_macros::quickcheck;

use std::fs;
use std::path::Path;

#[derive(Debug, PartialEq)]
pub struct FileMapping {
    i: String,
    o: String,
}

impl FileMapping {
    pub fn new(i: String, o: String) -> Self {
        Self { i, o }
    }

    pub fn display_line(&self) -> String {
        format!("{} -> {}", self.i.green(), self.o.green())
    }

    pub fn install(&self, force: bool) -> Result<String> {
        let i_path = Path::new(&self.i);
        let o_path = Path::new(&self.o);

        let copy_file = i_path.exists() && o_path.exists() && force;
        if copy_file {
            fs::copy(&self.i, &self.o)?;
            Ok(self.display_line())
        } else if o_path.exists() {
            Err(anyhow!(
                "{} exists and you didn't tell me to overwrite",
                o_path.display()
            ))
        } else {
            Err(anyhow!(
                "Unable to install from {} to {}",
                i_path.display(),
                o_path.display()
            ))
        }
    }

    pub fn uninstall(self) -> Result<String> {
        let o_path = Path::new(&self.o);

        if o_path.exists() && o_path.is_file() {
            fs::remove_file(o_path)?;
            Ok(format!("Removed {}", o_path.display()))
        } else {
            Ok(format!("Unable to remove {}", o_path.display()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[quickcheck]
    fn test_display_line(a: String, b: String) -> bool {
        let file_mapping = FileMapping {
            i: a.clone(),
            o: b.clone(),
        };
        let display_line = file_mapping.display_line();

        display_line.contains(&a) && display_line.contains(&b) && display_line.contains(" -> ")
    }
}
