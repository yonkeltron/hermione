use anyhow::{anyhow, Result};
use directories::ProjectDirs;
use fs_extra::dir;

use std::path::{Path, PathBuf};

use crate::downloaded_package::DownloadedPackage;

const QUALIFIER: &str = "dev";
const ORGANIZATION: &str = "hermione";
const APPLICATION: &str = "herm";

pub struct PackageService {
    pub project_dirs: ProjectDirs,
}

impl PackageService {
    pub fn new() -> Result<Self> {
        Ok(PackageService {
            project_dirs: Self::project_dirs()?,
        })
    }

    pub fn download_dir(&self) -> PathBuf {
        self.project_dirs.cache_dir().to_path_buf()
    }

    pub fn install_dir(&self) -> PathBuf {
        self.project_dirs.data_dir().to_path_buf()
    }

    pub fn installed_package_path(&self, package_name: &str) -> Result<PathBuf> {
        let path = self.install_dir().join(package_name);
        if path.is_dir() {
            Ok(path)
        } else {
            Err(anyhow!("It appears that {} isn't installed.", package_name))
        }
    }

    pub fn project_dirs() -> Result<ProjectDirs> {
        match ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION) {
            Some(pd) => Ok(pd),
            None => Err(anyhow!("Unable to determine directory structure.")),
        }
    }

    pub fn download(src: String) -> Result<DownloadedPackage> {
        let package = PackageService {
            project_dirs: Self::project_dirs()?,
        };
        let path = Path::new(&src).canonicalize()?;
        let package_name = Self::source_to_package_name(&src);
        let checkout_path = package.download_dir().join(&package_name);

        if path.is_dir() {
            println!(
                "Copying Package {} to {}",
                path.display(),
                checkout_path.display()
            );
            let options = dir::CopyOptions::new();
            dir::copy(&path, &checkout_path, &options)?;
            let local_path = checkout_path;
            Ok(DownloadedPackage {
                local_path: Path::new(&local_path).to_path_buf(),
                package_name,
                package_service: package,
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
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs;

    #[test]
    fn test_source_to_package_name_with_url() {
        let input = "http://github.com/panda/bamboo.git";
        let expected = String::from("bamboo");

        assert_eq!(PackageService::source_to_package_name(input), expected);
    }

    #[test]
    fn test_source_to_package_name_with_local_path() {
        let input = "./panda";
        let expected = String::from("panda");

        assert_eq!(PackageService::source_to_package_name(input), expected);
    }

    #[test]
    fn test_download() {
        let src = String::from("./example-package");

        let package = PackageService::download(src).expect("Unable to instantiate package");
        assert!(package.local_path.is_dir());
        fs::remove_dir_all(package.local_path).expect("Unable to remove package in test");
    }

    #[test]
    fn test_install_path() {
        let package_name = "panda";

        let package = PackageService::new().expect("Could not create package service");
        let actual = package
            .installed_package_path(package_name)
            .expect("Package is not installed");

        let expected = Path::new("panda").to_path_buf();

        assert_eq!(expected, actual);
    }
}
