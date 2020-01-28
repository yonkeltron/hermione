use anyhow::Result;
use fs_extra::dir;
use git2::Repository;

use std::env;
use std::fs;
use std::path::Path;

use crate::config::Config;
use crate::manifest::Manifest;

pub struct Package {
    pub local_path: String,
    pub source: String,
}

impl Package {
    pub fn new_from_source(src: String, config: Config) -> Result<Package> {
        let path = Path::new(&src).canonicalize()?;
        let package_name = Self::source_to_package_name(&src);
        let checkout_path = format!("{}/{}", &config.hermione_home, package_name);
        let dest_path = Path::new(&checkout_path);

        let local_path = if path.is_dir() {
            let options = dir::CopyOptions::new();
            dir::copy(&path, &config.hermione_home, &options)?;
            checkout_path
        } else {
            let repo = Repository::clone(&src, dest_path)?;
            let package_name = repo.path().display();
            format!("{}", package_name)
        };

        Ok(Package {
            local_path: local_path,
            source: src,
        })
    }

    fn source_to_package_name(src: &str) -> String {
        let path = Path::new(src);

        let package_name = match path.file_stem() {
            Some(stem) => String::from(stem.to_string_lossy()),
            None => String::from("UNKNOWN_PACKAGE"),
        };

        package_name
    }

    pub fn install(self) -> Result<usize> {
        let manifest = Manifest::new_from_file(format!("{}/hermione.yml", &self.local_path))?;

        let mapping_length = manifest.mappings.len();

        for mapping in manifest.mappings {
            let activity_line = mapping.install(false)?;
            println!("{}", activity_line);
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

    pub fn remove(&self) -> Result<usize> {
        let files_removed = self.uninstall()?;

        fs::remove_dir_all(&self.local_path)?;

        Ok(files_removed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_to_package_name_with_url() {
        let input = "http://github.com/panda/bamboo.git";
        let expected = String::from("bamboo");

        assert_eq!(Package::source_to_package_name(input), expected);
    }

    #[test]
    fn test_source_to_package_name_with_local_path() {
        let input = "./panda";
        let expected = String::from("panda");

        assert_eq!(Package::source_to_package_name(input), expected);
    }

    #[test]
    fn test_from_source_with_local() {
        let src = String::from("./example-package");

        let config = Config::load().expect("Unable to load config");
        config
            .init_hermione_home()
            .expect("Unable to init Hermione home in test");

        let package = Package::new_from_source(src, config).expect("Unable to instantiate package");

        let local_path = package.local_path.clone();

        assert!(Path::new(&local_path).is_dir());

        package.remove().expect("Unable to clean up after test");

        assert!(!Path::new(&local_path).is_dir());
    }
}
