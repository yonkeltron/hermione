use color_eyre::eyre::{eyre, Result};
use serde::{Deserialize, Serialize};
use ssri::{Integrity, IntegrityChecker};
use tera::{Context, Tera};

use std::fs;
use std::path::{Path, PathBuf};

use crate::file_mapping::FileMapping;
use crate::package_service::PackageService;

#[cfg(target_family = "unix")]
const PLATFORM: &str = "unix";

#[cfg(target_family = "windows")]
const PLATFORM: &str = "windows";

/// Mapping Definitions are where you put the input `i` files and the output `o` location
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct FileMappingDefinition {
    /// Input file path - Where is the desired file in the package.
    pub i: String,
    /// Output file path - Where you would like it to go on the system.
    pub o: String,
    /// Specifies file mapping to occur only when matching platform
    pub platform: Option<String>,
    /// Subresource Integrity (SRI) according to https://w3c.github.io/webappsec-subresource-integrity/
    pub integrity: Option<String>,
}

impl FileMappingDefinition {
    /// Return a new FileMappingDefinition.
    ///
    /// ### Arguments
    ///
    /// * `i` - `String` input file path.
    /// * `o` - `String` output file path.
    pub fn new(i: String, o: String, platform: Option<String>, integrity: Option<String>) -> Self {
        FileMappingDefinition {
            i,
            o,
            platform,
            integrity,
        }
    }

    /// Returns a FileMapping.
    ///
    /// ### Arguments
    ///
    /// * package_service - Borrowed reference to PackageService.
    /// * package_path_buf - Root path of your package.
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
            Err(e) => Err(eyre!(
                "Unable to calculate file mapping {} because {}",
                self.o,
                e.to_string()
            )),
        }
    }

    /// Returns true if the file mapping is for the running platform family
    pub fn valid_platform_family(&self) -> bool {
        match self.platform.as_ref() {
            Some(platform) => platform == PLATFORM,
            None => true,
        }
    }

    pub fn verify_integrity(&self, directory_location: PathBuf) -> Result<bool> {
        match &self.integrity {
            Some(checksum) => {
                let parsed: Integrity = checksum.parse()?;
                let mut checker = IntegrityChecker::new(parsed);
                let file_contents = fs::read(directory_location.join(&self.i))?;
                checker.input(&file_contents);

                Ok(checker.result().is_ok())
            }
            None => Ok(false),
        }
    }

    pub fn set_integrity(&mut self, package_path: PathBuf) -> Result<String> {
        let file_path = package_path.join(&self.i);
        let file_contents = fs::read(file_path)?;

        let sri = Integrity::from(&file_contents);

        self.integrity = Some(sri.to_string());

        let integrity_string = sri.to_string();

        Ok(integrity_string)
    }
}
