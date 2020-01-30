use anyhow::Result;

use std::fs;
use std::path::PathBuf;

use crate::manifest::Manifest;

pub struct InstalledPackage {
    pub local_path: PathBuf,
}

impl InstalledPackage {
    pub fn uninstall(&self) -> Result<bool> {
        let manifest_path = self.local_path.join("hermione.yml");
        let manifest_path_string = format!("{}", manifest_path.display());
        let manifest = Manifest::new_from_file(&manifest_path_string)?;

        for mapping in manifest.mappings {
            mapping.uninstall()?;
        }

        Ok(true)
    }

    pub fn remove(&self) -> Result<bool> {
        let files_removed = self.uninstall()?;

        fs::remove_dir_all(&self.local_path)?;

        Ok(true)
    }
}
