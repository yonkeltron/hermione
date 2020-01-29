use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use std::fs;
use std::path::Path;

use crate::file_mapping::FileMapping;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Manifest {
    pub name: String,
    pub author: String,
    pub mappings: Vec<FileMapping>,
}

impl Manifest {
    pub fn new_from_file(path_to_manifest_file: String) -> Result<Manifest> {
        let path = Path::new(&path_to_manifest_file);

        if path.is_file() {
            let yaml = fs::read_to_string(path)?;
            let manifest: Manifest = serde_yaml::from_str(&yaml)?;

            Ok(manifest)
        } else {
            Err(anyhow!(
                "Looks like {} is not a file",
                path_to_manifest_file
            ))
        }
    }
}
