use anyhow::Result;

use std::fs;
use std::path::Path;

use crate::manifest::Manifest;

pub struct InstalledPackage {
    local_path: Path,
}

impl InstalledPackage {
    pub fn uninstall(&self) -> Result<usize> {
        let manifest =
            Manifest::new_from_file(format!("{}/hermione.yml", &self.local_path.display()))?;

        let mapping_length = manifest.mappings.len();

        for mapping in manifest.mappings {
            mapping.uninstall()?;
        }

        Ok(mapping_length)
    }

    pub fn remove(&self) -> Result<usize> {
        let files_removed = self.uninstall()?;

        fs::remove_dir_all(&self.local_path)?;

        Ok(files_removed)
    }
}
