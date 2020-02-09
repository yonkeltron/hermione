use anyhow::{anyhow, Result};
use directories::ProjectDirs;
use fs_extra::dir;
use git2::Repository;

use std::path::{Path, PathBuf};

use crate::config::Config;
use crate::downloaded_package::DownloadedPackage;

pub struct Package {}

const QUALIFIER: &str = "dev";
const ORGANIZATION: &str = "hermione";
const APPLICATION: &str = "herm";

impl Package {
    pub fn download_dir() -> Result<PathBuf> {
        match ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION) {
            Some(pd) => Ok(pd.cache_dir().to_path_buf()),
            None => Err(anyhow!("Unable to determine directory structure.")),
        }
    }

    pub fn download(src: String, config: &Config) -> Result<DownloadedPackage> {
        let path = Path::new(&src).canonicalize()?;
        let package_name = Self::source_to_package_name(&src);
        let checkout_path = Self::download_dir()?.join(&package_name);

        if path.is_dir() {
            println!(
                "Copying Package {} to {}",
                path.display(),
                checkout_path.display()
            );
            let options = dir::CopyOptions::new();
            dir::copy(&path, &config.hermione_home, &options)?;
            let local_path = checkout_path;
            Ok(DownloadedPackage {
                local_path: Path::new(&local_path).to_path_buf(),
                package_name,
            })
        } else {
            // let repo = Repository::clone(&src, dest_path)?;
            // repo.path().to_path_buf()
            Err(anyhow!(
                "Path to package does not exist: {}",
                path.display()
            ))
        }
    }

    fn source_to_package_name(src: &str) -> String {
        let path = Path::new(src);

        match path.file_stem() {
            Some(stem) => String::from(stem.to_string_lossy()),
            None => String::from("UNKNOWN_PACKAGE"),
        }
    }

    pub fn install_path(hermione_home: &str, package_name: &str) -> PathBuf {
        Path::new(hermione_home).join(package_name).to_path_buf()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs;

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
    fn test_download() {
        let src = String::from("./example-package");

        let config = Config::load().expect("Unable to load config in test");
        config
            .init_hermione_home()
            .expect("Unable to init Hermione home in test");

        let package = Package::download(src, &config).expect("Unable to instantiate package");
        assert!(package.local_path.is_dir());
        fs::remove_dir_all(package.local_path).expect("Unable to remove package in test");
    }

    #[test]
    fn test_install_path() {
        let package_name = "panda";
        let hermione_home = "bamboo";

        let actual = Package::install_path(hermione_home, package_name);

        let expected = Path::new("bamboo/panda").to_path_buf();

        assert_eq!(expected, actual);
    }
}
