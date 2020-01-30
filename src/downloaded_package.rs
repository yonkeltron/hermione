use anyhow::{anyhow, Result};

use std::fs;
use std::path::Path;

use crate::installed_package::InstalledPackage;
use crate::manifest::Manifest;

pub struct DownloadedPackage {
    local_path: Path,
}

impl DownloadedPackage {
    pub fn install(self) -> Result<InstalledPackage> {
        let manifest = Manifest::new_from_file(format!("{}/hermione.yml", &self.local_path))?;

        for mapping in manifest.mappings {
            let activity_line = mapping.install(false)?;
            println!("{}", activity_line);
        }

        Ok(InstalledPackage {
            local_path: self.local_path,
        });
    }
}
