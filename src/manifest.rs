use color_eyre::eyre::{eyre, Result, WrapErr};
use serde::{Deserialize, Serialize};

use std::fs;
use std::io;
use std::path::PathBuf;

use crate::file_mapping_definition::FileMappingDefinition;
use crate::hooks::Hooks;

const MANIFEST_FILE_NAME: &str = "hermione.yml";

/// Manifest represents the definition of your Hermione package.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Manifest {
    /// Name of your package 📦.
    pub name: String,
    /// Your name 😀.
    pub authors: Vec<String>,
    /// A description of your package, short sweet and to the point.
    pub description: String,
    /// A unique "reverse domain name" identifier for your package
    pub id: String,
    /// Mappings define the core operation of Hermione.
    /// Here is where you define the `what` and the `where`
    /// . The what being the file you want to move and the where being where do you want to move it.
    pub mappings: Vec<FileMappingDefinition>,

    pub hooks: Option<Hooks>,
}

impl Manifest {
    /// Generates a Manifest struct from a given `hermione.yml` path.
    pub fn new_from_path(path: PathBuf) -> Result<Manifest> {
        if path.is_file() {
            let yaml = fs::read_to_string(path)?;
            let manifest: Manifest =
                serde_yaml::from_str(&yaml).wrap_err("Could not parse manifest yaml")?;

            Ok(manifest)
        } else {
            Err(eyre!("Looks like {} is not a file", path.display()))
        }
    }

    pub fn set_mappings(self, file_mapping_definitions: Vec<FileMappingDefinition>) -> Self {
        Self {
            mappings: file_mapping_definitions,
            ..self
        }
    }

    /// Generates a Manifest struct from a given `hermione.yml` reader data.
    pub fn new_from_reader<R>(data: R) -> Result<Manifest>
    where
        R: io::Read,
    {
        Ok(serde_yaml::from_reader(data)?)
    }

    pub fn manifest_file_name() -> String {
        String::from(MANIFEST_FILE_NAME)
    }
}
