use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use std::fs;
use std::path::PathBuf;

use crate::file_mapping_definition::FileMappingDefinition;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Manifest {
    pub name: String,
    pub author: String,
    pub description: String,
    pub mappings: Vec<FileMappingDefinition>,
}

impl Manifest {
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
