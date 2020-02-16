use anyhow::{anyhow, Context, Result};
use directories::{BaseDirs, ProjectDirs};
use fs_extra::dir;

use std::fs;
use std::path::{Path, PathBuf};

use crate::downloaded_package::DownloadedPackage;
use crate::installed_package::InstalledPackage;

const QUALIFIER: &str = "dev";
const ORGANIZATION: &str = "hermione";

#[cfg(test)]
const APPLICATION: &str = "herm_test";

#[cfg(not(test))]
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

    pub fn home_dir(&self) -> Result<PathBuf> {
        match BaseDirs::new() {
            Some(base_dirs) => Ok(base_dirs.home_dir().to_path_buf()),
            None => Err(anyhow!("Unable to find HOME directory")),
        }
    }

    pub fn installed_package_path(&self, package_name: &str) -> Result<PathBuf> {
        let path = self.install_dir().join(package_name);
        if path.is_dir() && !package_name.trim().is_empty() {
            Ok(path)
        } else if package_name.trim().is_empty() {
            Err(anyhow!("Package name can not be empty."))
        } else {
            Err(anyhow!("It appears that {} isn't installed.", package_name))
        }
    }

    pub fn list_installed_packages(&self) -> Result<Vec<InstalledPackage>> {
        if !self.install_dir().exists() {
            let r: Vec<InstalledPackage> = Vec::new();
            Ok(r)
        } else {
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
        let package = PackageService::new()?;
        let path = Path::new(&src).canonicalize()?;
        let package_name = Self::source_to_package_name(&src);
        let checkout_path = package.download_dir();

        if !checkout_path.exists() {
            println!("Creating download directory {}", &checkout_path.display());
            dir::create_all(&checkout_path, false)?;
        }
        if path.is_dir() {
            println!(
                "Copying Package {} to {}",
                path.display(),
                checkout_path.display()
            );
            let mut options = dir::CopyOptions::new();
            options.copy_inside = true;
            options.overwrite = true;
            dir::copy(&path, &checkout_path, &options)
                .with_context(|| format!("Error copying package to {}", checkout_path.display()))?;

            Ok(DownloadedPackage {
                local_path: checkout_path.join(&package_name),
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

    #[cfg(test)]
    fn purge_installed_packages(&self) -> Result<()> {
        let errored_uninstalled = self
            .list_installed_packages()?
            .into_iter()
            .map(|installed_package| installed_package.uninstall().unwrap_or(false))
            .filter(|was_removed| !was_removed)
            .collect::<Vec<bool>>();

        if errored_uninstalled.is_empty() {
            Ok(())
        } else {
            Err(anyhow!("Failed to uninstall all packges."))
        }
    }

    #[cfg(test)]
    fn purge_everything(&self) -> Result<()> {
        match self.purge_installed_packages() {
            Ok(_) => {
                println!("All packages have been uninstalled.");
                println!(
                    "Removing install directory => ({})",
                    self.install_dir().display()
                );
                if self.install_dir().exists() {
                    fs::remove_dir_all(self.install_dir())?
                }
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                eprintln!("WARNING: Install dir has not been removed")
            }
        }
        println!(
            "Removing download dir => ({})",
            self.download_dir().display()
        );
        if self.download_dir().exists() {
            fs::remove_dir_all(self.download_dir())?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs;

    fn purge() {
        let package_service =
            PackageService::new().expect("Unable to instantiate PackageService in test");
        package_service
            .purge_everything()
            .expect("Failed to clean up after tests");
    }

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
        let package_name = "example-package";

        let package_service: PackageService =
            PackageService::new().expect("Could not create package service");
        let installed_package =
            PackageService::download_and_install("./example-package".to_string())
                .expect("Failed to install package");

        let actual = package_service
            .installed_package_path(package_name)
            .expect("Package is not installed");
        installed_package.remove().expect("Failed to clean up dir");

        let expected = package_service.install_dir().join(&package_name);
        assert_eq!(expected, actual);
    }
}
