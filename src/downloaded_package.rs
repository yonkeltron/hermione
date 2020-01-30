use anyhow::Result;

use std::fs;
use std::path::PathBuf;

use crate::installed_package::InstalledPackage;
use crate::manifest::Manifest;

pub struct DownloadedPackage {
    pub local_path: PathBuf,
}

impl DownloadedPackage {
    pub fn install(self) -> Result<InstalledPackage> {
        let manifest_path = self.local_path.join("hermione.yml");
        let manifest_path_string = format!("{}", manifest_path.display());
        let manifest = Manifest::new_from_file(&manifest_path_string)?;

        for mapping in manifest.mappings {
            let activity_line = mapping.install(false)?;
            println!("{}", activity_line);
        }

        Ok(InstalledPackage {
            local_path: self.local_path,
        })
    }

    pub fn remove(self) -> Result<()> {
        fs::remove_dir_all(self.local_path)?;
        Ok(())
    }
}
