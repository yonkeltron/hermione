use anyhow::{anyhow, Context, Result};
use directories::ProjectDirs;
use fs_extra::dir;

use std::fs;
use std::path::{Path, PathBuf};

use crate::downloaded_package::DownloadedPackage;
use crate::installed_package::InstalledPackage;

const QUALIFIER: &str = "dev";
const ORGANIZATION: &str = "hermione";
const APPLICATION: &str = "herm";

#[derive(Clone, Debug)]
pub struct PackageService {
    pub project_dirs: ProjectDirs,
}

impl PackageService {
    pub fn new() -> Result<Self> {
        Ok(PackageService {
            project_dirs: Self::project_dirs()?,
        })
    }

    pub fn init(&self) -> Result<bool> {
        let d_dir = self.download_dir();
        println!("Creating download directory {}", &d_dir.display());
        fs::create_dir_all(&d_dir)
            .with_context(|| format!("Unable to create download directory {}", d_dir.display()))?;

        let i_dir = self.install_dir();
        println!("Creating install directory {}", &i_dir.display());
        fs::create_dir_all(&i_dir)
            .with_context(|| format!("Unable to create install directory {}", i_dir.display()))?;
        Ok(true)
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

    pub fn list_installed_packages(&self) -> Result<Vec<InstalledPackage>> {
        let mut entries = fs::read_dir(self.install_dir())?
            .map(|entry_result| entry_result.map(|entry| entry.path()))
            .collect::<Result<Vec<_>, std::io::Error>>()?;
        entries.sort();
        let dirs = entries.iter().filter(|entry_path| entry_path.is_dir());
        Ok(dirs
            .map(|entry| {
                let package_name = String::from(entry.to_string_lossy());
                let package_service = self.clone();
                let local_path = entry.clone();
                InstalledPackage {
                    local_path,
                    package_name: PackageService::source_to_package_name(&package_name),
                    package_service,
                }
            })
            .collect())
    }

    pub fn project_dirs() -> Result<ProjectDirs> {
        match ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION) {
            Some(pd) => Ok(pd),
            None => Err(anyhow!("Unable to determine directory structure.")),
        }
    }

    pub fn download_and_install(src: String) -> Result<InstalledPackage> {
        let downloaded_package = Self::download(src)?;
        Ok(downloaded_package.install()?)
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
    fn test_download_and_install() {
        let package_service =
            PackageService::new().expect("Unable to instantiate PackageService in test");
        package_service
            .init()
            .expect("Unable to initialize directories in test");
        let installed_package_list = package_service
            .list_installed_packages()
            .expect("Can not get list of installed packages in test");
        assert_eq!(0, installed_package_list.len());
    }

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
