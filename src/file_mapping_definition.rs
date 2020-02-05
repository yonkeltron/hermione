use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use tera::{Context, Tera};

use crate::file_mapping::FileMapping;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct FileMappingDefinition {
    i: String,
    o: String,
}

impl FileMappingDefinition {
    pub fn render_file_mapping(self) -> Result<FileMapping> {
        let context = Context::new();
        match Tera::one_off(&self.o, &context, true) {
            Ok(o) => Ok(FileMapping::new(self.i, o)),
            Err(e) => Err(anyhow!(
                "Unable to calculate file mapping {} because {}",
                self.o,
                e.to_string()
            )),
        }
    }
}
