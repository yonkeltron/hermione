use anyhow::{anyhow, Context, Result};

#[cfg(test)]
use quickcheck_macros::quickcheck;

use std::fs;
use std::path::PathBuf;
/// Describes the file mapping between input `i` and output `o`.
/// This struct is responsible for installing and uninstalling a file.
#[derive(Debug, PartialEq)]
pub struct FileMapping {
    /// input PathBuf - What the desired file.
    i: PathBuf,
    /// output PathBuf - Where you would like it to go.
    o: PathBuf,
}

impl FileMapping {
    /// Returns an instance of FileMapping
    ///
    /// ### Arguments
    ///
    /// * i - `PathBuf` of the input file path
    /// * o - `PathBuf` of the output file path
    ///
    pub fn new(i: PathBuf, o: PathBuf) -> Self {
        Self { i, o }
    }

    /// Returns String print out of File Mapping
    pub fn display_line(&self) -> String {
        format!(
            "Copying {} -> {}",
            self.i.to_string_lossy(),
            self.o.to_string_lossy(),
        )
    }

    /// Installs the input file to the output path
    ///
    /// Returns String Result
    pub fn install(&self) -> Result<String> {
        let copy_file = self.i.exists() && !self.o.exists();
        match self.o.parent() {
            Some(parent_path) => {
                if !parent_path.exists() {
                    fs::create_dir_all(parent_path)?;
                }
            }
            None => {}
        };
        if copy_file {
            fs::copy(&self.i, &self.o).with_context(|| {
                format!(
                    "Failed to copy file {} -> {}",
                    self.i.display(),
                    self.o.display()
                )
            })?;
            Ok(self.display_line())
        } else if self.o.exists() {
            Err(anyhow!(
                "{} exists and Hermoine will not overwrite it.",
                self.o.display()
            ))
        } else {
            Err(anyhow!(
                "Unable to install from {} -> {}",
                self.i.display(),
                self.o.display()
            ))
        }
    }

    /// Uninstalls the output path defined in the manifest
    ///
    /// Returns String Result
    pub fn uninstall(self) -> Result<String> {
        if self.o.is_file() {
            fs::remove_file(&self.o)?;
            Ok(format!("Removed {}", self.o.display()))
        } else {
            Ok(format!(
                "Not removing {} because it is not a file",
                self.o.display()
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[quickcheck]
    fn test_display_line(a: String, b: String) -> bool {
        let file_mapping = FileMapping {
            i: Path::new(&a).to_path_buf(),
            o: Path::new(&b).to_path_buf(),
        };
        let display_line = file_mapping.display_line();

        display_line.contains(&a) && display_line.contains(&b) && display_line.contains(" -> ")
    }
}
