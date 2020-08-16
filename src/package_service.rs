use anyhow::{anyhow, Context, Result};
use directories::{BaseDirs, ProjectDirs};
use fs_extra::dir;
use lockfile::Lockfile;
use paris::Logger;

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process;

use crate::downloaded_package::DownloadedPackage;
use crate::git_downloader::GitDownloader;
use crate::installed_package::InstalledPackage;

const QUALIFIER: &str = "dev";
const ORGANIZATION: &str = "hermione";

#[cfg(test)]
const APPLICATION: &str = "herm_test";

#[cfg(not(test))]
const APPLICATION: &str = "herm";

/// PackageService provides some of the core logic around initializing `hermione`.
/// It is responsible for wrapping a logger for subsequent actions to use,
/// providing information for OS specific file directories and provide
/// entry points for initiating the install and remove actions.
#[derive(Clone, Debug)]
pub struct PackageService {
    pub project_dirs: ProjectDirs,
}

impl PackageService {
    /// Create a new PackageService.
    ///
    /// ### Arguments
    ///
    /// * logger - Instance of Logger.
    ///
    /// Returns an instance of PackageService as a Result.
    pub fn new() -> Result<Self> {
        Ok(PackageService {
            project_dirs: Self::project_dirs()?,
        })
    }

    /// Creates the download and install dir for the respective OS.
    ///
    /// Returns a bool as a Result.
    pub fn init(&self) -> Result<bool> {
        let mut logger = Logger::new();
        let d_dir = self.download_dir();
        if !d_dir.is_dir() {
            logger.info(format!("Creating download directory {}", &d_dir.display()));
            fs::create_dir_all(&d_dir).with_context(|| {
                format!("Unable to create download directory {}", d_dir.display())
            })?;
        }

        let i_dir = self.install_dir();
        if !i_dir.is_dir() {
            logger.info(format!("Creating install directory: {}", &i_dir.display(),));
            fs::create_dir_all(&i_dir).with_context(|| {
                format!("Unable to create install directory {}", i_dir.display())
            })?;
        }

        Ok(true)
    }

    /// Returns a PathBuf to the download directory for the respective OS.
    pub fn download_dir(&self) -> PathBuf {
        self.project_dirs.cache_dir().to_path_buf()
    }

    /// Returns a PathBuf to the install directory for the respective OS.
    pub fn install_dir(&self) -> PathBuf {
        self.project_dirs.data_dir().to_path_buf()
    }

    /// Returns a lockfile path
    pub fn lockfile(&self) -> Result<Lockfile, anyhow::Error> {
        let lockfile_name = "hermione.lock";
        let lockfile_path = self.install_dir().join(lockfile_name);

        match Lockfile::create(&lockfile_path) {
            Ok(mut lockfile) => {
                let proc_id = format!("{}", process::id());
                lockfile.write_all(proc_id.as_bytes()).with_context(|| {
                    format!(
                        "Unable to write PID to lockfile at {}",
                        lockfile_path.display()
                    )
                })?;

                Ok(lockfile)
            }
            Err(err) => Err(anyhow!(
                "Is Hermione already running? Unable to obtain lockfile at {} because: {}",
                lockfile_path.display(),
                err
            )),
        }
    }

    /// Returns a PathBuf to the users home directory for their respective OS.
    pub fn home_dir(&self) -> Result<PathBuf> {
        match BaseDirs::new() {
            Some(base_dirs) => Ok(base_dirs.home_dir().to_path_buf()),
            None => Err(anyhow!("Unable to find HOME directory")),
        }
    }

    /// Gets an instance of an installed package if one exists.
    ///
    /// ### Arguments
    ///
    /// * package_name - The package name must not be the path, but rather the name you called the hermione folder
    ///
    /// Returns an InstalledPackage as a Result.
    pub fn get_installed_package(self, package_name: String) -> Result<InstalledPackage> {
        let package_path = self.installed_package_path(&package_name)?;

        Ok(InstalledPackage {
            local_path: package_path,
            package_name,
            package_service: self,
        })
    }

    /// Gets a PathBuf of an installed package if one exists.
    ///
    /// ### Arguments
    ///
    /// * package_name - The package name must not be the path, but rather the name you called the hermione folder
    ///
    /// Returns an PathBuf as a Result.
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

    /// Returns a vector of InstalledPackage as a Result
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

    /// This returns a ProjectDirs object for the respective OS.
    /// We avoid hard coding OS specific information when dealing with Paths and rely
    /// on the [directories](https://crates.io/crates/directories) to get the respective paths.
    ///
    /// Returns an instance of ProjectDirs as a Result.
    pub fn project_dirs() -> Result<ProjectDirs> {
        match ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION) {
            Some(pd) => Ok(pd),
            None => Err(anyhow!("Unable to determine directory structure.")),
        }
    }

    /// Initiate a download and install action for a given Hermione package location.
    ///
    /// ### Arguments
    ///
    /// * src - Location of the Hermione package as a local path.
    ///
    /// Returns an InstalledPackage as a Result.
    pub fn download_and_install(self, src: String) -> Result<InstalledPackage> {
        let downloaded_package = self.download(src)?;
        Ok(downloaded_package.install()?)
    }

    /// Initiate a download action for a given Hermione package location.
    ///
    /// ### Arguments
    ///
    /// * src - Location of the Hermione package as a local path.
    ///
    /// Returns an DownloadedPackage as a Result.
    pub fn download(self, src: String) -> Result<DownloadedPackage> {
        let package_name = Self::source_to_package_name(&src);
        let checkout_path = self.download_dir();
        let mut logger = paris::Logger::new();
        if !checkout_path.exists() {
            logger.info(format!(
                "Creating download directory at {}",
                &checkout_path.display()
            ));
            dir::create_all(&checkout_path, false)?;
        }

        if src.ends_with("git") {
            logger.info("Detected remote package");
            let clone_path = checkout_path.join(&package_name);
            let git_downloader = GitDownloader::new(clone_path, package_name, self);
            git_downloader.download_or_update(src)
        } else {
            let path = Path::new(&src).canonicalize()?;
            if path.is_dir() {
                logger.info(format!(
                    "Copying Package {} -> {}",
                    checkout_path.display(),
                    path.display(),
                ));
                let mut options = dir::CopyOptions::new();
                options.copy_inside = true;
                options.overwrite = true;
                dir::copy(&path, &checkout_path, &options).with_context(|| {
                    format!("Error copying package to {}", checkout_path.display())
                })?;

                Ok(DownloadedPackage {
                    local_path: checkout_path.join(&package_name),
                    package_name,
                    package_service: self,
                })
            } else {
                Err(anyhow!(
                    "Path to package does not exist: {}",
                    path.display()
                ))
            }
        }
    }

    /// Parses out a src string into a Path and grabs the stem to get the Hermione package name.
    fn source_to_package_name(src: &str) -> String {
        let path = Path::new(src);

        match path.file_stem() {
            Some(stem) => String::from(stem.to_string_lossy()),
            None => String::from("UNKNOWN_PACKAGE"),
        }
    }

    /// Purge all installed packages, this will uninstall all installed packages and then remove the install directory.
    ///
    /// If one package fails to uninstall then the install directory is not removed but is left for the owner to
    /// remove at their own discretion as it requires attention to fully delete.
    /// This will normally be a problem due to file permission changes since the package was installed,
    /// or more likely that a file defined in the hermione package was manually removed or altered after the fact.
    ///
    /// Returns an empty Result.
    pub fn purge_installed_packages(&self) -> Result<()> {
        let mut logger = Logger::new();
        logger.info("Started removing all installed packages");
        let errored_uninstalled = self
            .list_installed_packages()?
            .into_iter()
            .map(|installed_package| {
                logger.info(format!(
                    "Removing package: {}",
                    installed_package.package_name.clone()
                ));
                installed_package.remove().unwrap_or(false)
            })
            .filter(|was_removed| !was_removed)
            .collect::<Vec<bool>>();

        if errored_uninstalled.is_empty() {
            Ok(())
        } else {
            Err(anyhow!("Failed to uninstall all packges"))
        }
    }

    /// Implode will call `purge_installed_packages` and then if all was successful it will remove the download directory.
    ///
    /// Returns an empty Result.
    pub fn implode(&self) -> Result<()> {
        let mut logger = paris::Logger::new();
        match self.purge_installed_packages() {
            Ok(_) => {
                logger.info("All packages have been uninstalled.");
                logger.info(format!(
                    "Removing install directory: {}",
                    self.install_dir().display(),
                ));
                if self.install_dir().exists() {
                    fs::remove_dir_all(self.install_dir())?
                }
            }
            Err(e) => {
                logger.error(format!(
                    "Error deleting installed packages and installed directory because {}",
                    e.to_string(),
                ));
            }
        }
        logger.info(format!(
            "Removing download directory: {}",
            self.download_dir().display()
        ));
        if self.download_dir().exists() {
            fs::remove_dir_all(self.download_dir())?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use quickcheck_macros::quickcheck;
    use scopeguard::defer;

    use std::fs;

    fn purge() {
        let package_service =
            PackageService::new().expect("Unable to instantiate PackageService in test");
        package_service
            .implode()
            .expect("Failed to clean up in test");
    }

    #[test]
    fn test_list_installed_packages_with_nothing() {
        defer!(purge());
        let package_service =
            PackageService::new().expect("Unable to instantiate PackageService in test");
        let installed_package_list = package_service
            .list_installed_packages()
            .expect("Can not get list of installed packages in test");
        assert_eq!(0, installed_package_list.len());
    }

    #[test]
    fn test_list_installed_packages_with_package() {
        defer!(purge());

        let src = String::from("./example-package");
        let package_service =
            PackageService::new().expect("Unable to instantiate PackageService in test");
        package_service
            .download_and_install(src)
            .expect("Unable to instantiate package in test");

        let test_package_service =
            PackageService::new().expect("Unable to instantiate PackageService in test");
        let installed_package_list = test_package_service
            .list_installed_packages()
            .expect("Can not get list of installed packages in test");
        assert_eq!(1, installed_package_list.len());
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
        defer!(purge());

        let src = String::from("./example-package");
        let package_service =
            PackageService::new().expect("Unable to instantiate PackageService in test");
        let package = package_service
            .download(src)
            .expect("Unable to instantiate package in test");
        assert!(package.local_path.is_dir());
        fs::remove_dir_all(package.local_path).expect("Unable to remove package in test");
    }

    #[test]
    fn test_download_and_install() {
        defer!(purge());

        let src = String::from("./example-package");
        let package_service =
            PackageService::new().expect("Unable to instantiate PackageService in test");

        package_service
            .download_and_install(src)
            .expect("Unable to instantiate package in test");

        let test_package_service =
            PackageService::new().expect("Unable to instantiate PackageService in test");

        let installed_path = test_package_service
            .installed_package_path("example-package")
            .expect("Unable to remove example-packahe in test");
        assert!(installed_path.is_dir());
    }

    #[test]
    fn test_install_package_path() {
        defer!(purge());

        let package_name = "example-package";

        let package_service: PackageService =
            PackageService::new().expect("Could not create package service in test");

        package_service
            .download_and_install("./example-package".to_string())
            .expect("Failed to install package in test");

        let test_package_service =
            PackageService::new().expect("Unable to instantiate PackageService in test");

        let actual = test_package_service
            .installed_package_path(package_name)
            .expect("Package is not installed in test");

        let expected = test_package_service.install_dir().join(&package_name);
        assert_eq!(expected, actual);
    }

    #[quickcheck]
    fn from_package_name_with_bogus_package_always_fails(name: String) -> bool {
        let package_service: PackageService =
            PackageService::new().expect("Could not create package service in test");
        package_service.get_installed_package(name).is_err()
    }
}
