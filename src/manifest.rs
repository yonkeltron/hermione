use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use std::fs;
use std::path::PathBuf;

use crate::file_mapping_definition::FileMappingDefinition;

/// Manifest represents the definition of your Hermione package
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Manifest {
    /// Name of your package 📦.
    pub name: String,
    /// Your name 😀.
    pub author: String,
    /// A description of your package, short sweet and to the point.
    pub description: String,
    /// Mappings define the core operation of Hermione.
    /// Here is where you define the `what` and the `where`
    /// . The what being the file you want to move and the where being where do you want to move it.
    pub mappings: Vec<FileMappingDefinition>,
}

impl Manifest {
    /// Generates a Manifest struct from a given `hermione.yml` path
    pub fn new_from_path(path: PathBuf) -> Result<Manifest> {
        if path.is_file() {
            let yaml = fs::read_to_string(path)?;
            let manifest: Manifest = serde_yaml::from_str(&yaml)?;

            Ok(manifest)
        } else {
            Err(anyhow!("Looks like {} is not a file", path.display()))
        }
    }
}
