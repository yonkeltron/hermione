use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use tera::{Context, Tera};

use std::path::{Path, PathBuf};

use crate::file_mapping::FileMapping;
use crate::package_service::PackageService;

/// Mapping Definitions are where you put the input `i` files and the output `o` location
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct FileMappingDefinition {
    /// Input file path - What the desired file.
    i: String,
    /// Output file path - Where you would like it to go.
    o: String,
}

impl FileMappingDefinition {
    /// Return a new FileMappingDefinition
    ///
    /// ### Arguments
    ///
    /// * `i` - Input file path
    /// * `o` - Output file path
    pub fn new(i: String, o: String) -> Self {
        FileMappingDefinition { i, o }
    }

    /// Returns a FileMapping
    ///
    /// ### Arguments
    ///
    /// * package_service - Borrowed reference to PackageService
    /// * package_path_buf - Root path of your package
    pub fn render_file_mapping(
        self,
        package_service: &PackageService,
        package_path_buf: PathBuf,
    ) -> Result<FileMapping> {
        let mut context = Context::new();
        let home_dir_path_buf = package_service.home_dir()?;
        let home_dir = home_dir_path_buf.to_string_lossy();
        context.insert("HOME", &home_dir);
        match Tera::one_off(&self.o, &context, false) {
            Ok(o) => {
                let i_path = package_path_buf.join(&self.i);
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
