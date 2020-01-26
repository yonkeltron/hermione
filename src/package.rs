use anyhow::Result;
use fs_extra::dir;
use git2::Repository;

use std::fs;
use std::path::Path;

use crate::config::Config;
use crate::manifest::Manifest;

pub struct Package {
    pub local_path: String,
    pub source: String,
}

impl Package {
    pub fn new(src: String, config: Config) -> Result<Package> {
        let path = Path::new(&src);

        let checkout_path = format!("{}/{}", &config.hermione_home, &src);

        let local_path = if path.is_dir() {
            let options = dir::CopyOptions::new();
            dir::copy(&src, &checkout_path, &options)?;
            checkout_path
        } else {
            let repo = Repository::clone(&src, checkout_path)?;
            format!("{}", repo.path().display())
        };

        Ok(Package {
            local_path: local_path,
            source: src,
        })
    }

    pub fn install(self) -> Result<usize> {
        let manifest = Manifest::new_from_file(format!("{}/hermione.yml", &self.local_path))?;

        let mapping_length = manifest.mappings.len();

        for mapping in manifest.mappings {
            mapping.install(false)?;
        }

        Ok(mapping_length)
    }

    pub fn uninstall(&self) -> Result<usize> {
        let manifest = Manifest::new_from_file(format!("{}/hermione.yml", &self.local_path))?;

        let mapping_length = manifest.mappings.len();

        for mapping in manifest.mappings {
            mapping.uninstall()?;
        }

        Ok(mapping_length)
    }

    pub fn remove(self) -> Result<usize> {
        let files_removed = self.uninstall()?;

        fs::remove_dir_all(&self.local_path)?;

        Ok(files_removed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_with_local() {
        let src = String::from("./");

        let package = Package::new(src, confy::load("hermione").unwrap()).unwrap();

        assert!(Path::new(&package.local_path).is_dir());

        package.remove().expect("Unable to clean up after test");
    }
}
