use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use tera::{Context, Tera};

use std::path::Path;

use crate::file_mapping::FileMapping;
use crate::package_service::PackageService;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct FileMappingDefinition {
    i: String,
    o: String,
}

impl FileMappingDefinition {
    pub fn render_file_mapping(self, package_service: &PackageService) -> Result<FileMapping> {
        let mut context = Context::new();
        let home_dir_path_buf = package_service.home_dir()?;
        let home_dir = home_dir_path_buf.to_string_lossy();
        context.insert("HOME", &home_dir);
        match Tera::one_off(&self.o, &context, false) {
            Ok(o) => {
                let i_path = Path::new(&self.i).to_path_buf();
                let o_path = Path::new(&o).to_path_buf();
                Ok(FileMapping::new(i_path, o_path))
            }
            Err(e) => Err(anyhow!(
                "Unable to calculate file mapping {} because {}",
                self.o,
                e.to_string()
            )),
        }
    }
}
