use color_eyre::eyre::{eyre, Result, WrapErr};
use directories::{BaseDirs, ProjectDirs};
use fs_extra::dir;
use lockfile::Lockfile;
use paris::Logger;
use url::Url;

use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process;

use crate::downloaded_package::DownloadedPackage;
use crate::downloader::Downloader;
use crate::installed_package::InstalledPackage;
use crate::manifest::Manifest;
use crate::packer::Packer;
use crate::repositories::package_index::PackageIndex;

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
            fs::create_dir_all(&d_dir).wrap_err_with(|| {
                format!("Unable to create download directory {}", d_dir.display())
            })?;
        }

        let i_dir = self.install_dir();
        if !i_dir.is_dir() {
            logger.info(format!("Creating install directory: {}", &i_dir.display(),));
            fs::create_dir_all(&i_dir).wrap_err_with(|| {
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
    pub fn lockfile(&self) -> Result<Lockfile> {
        let lockfile_name = "hermione.lock";
        let lockfile_path = self.install_dir().join(lockfile_name);

        match Lockfile::create(&lockfile_path) {
            Ok(mut lockfile) => {
                let proc_id = format!("{}", process::id());
                lockfile.write_all(proc_id.as_bytes()).wrap_err_with(|| {
                    format!(
                        "Unable to write PID to lockfile at {}",
                        lockfile_path.display()
                    )
                })?;

                Ok(lockfile)
            }
            Err(err) => Err(eyre!(
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
            None => Err(eyre!("Unable to find HOME directory")),
        }
    }

    /// Gets an instance of an installed package if one exists.
    ///
    /// ### Arguments
    ///
    /// * package_name - The package name must not be the path, but rather the name you called the hermione folder
    ///
    /// Returns an InstalledPackage as a Result.
    pub fn get_installed_package(self, package_id: String) -> Result<InstalledPackage> {
        let package_path = self.installed_package_path(&package_id)?;
        let manifest_path = package_path.join(Manifest::manifest_file_name());
        let manifest = Manifest::new_from_path(manifest_path)?;
        Ok(InstalledPackage {
            local_path: package_path,
            manifest,
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
            Err(eyre!("Package name can not be empty."))
        } else {
            Err(eyre!("It appears that {} isn't installed.", package_name))
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
            let installed = dirs
                .map(|entry| {
                    let package_service = self.clone();
                    let local_path = entry.clone();
                    let manifest_path = local_path.join(Manifest::manifest_file_name());

                    match Manifest::new_from_path(manifest_path) {
                        Ok(manifest) => Some(InstalledPackage {
                            local_path,
                            manifest,
                            package_service,
                        }),
                        Err(_) => None,
                    }
                })
                .filter(|opt| opt.is_some())
                .map(|opt| opt.expect("Unable to read manifest from installed package"))
                .collect();
            Ok(installed)
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
            None => Err(eyre!("Unable to determine directory structure.")),
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
        let download_dir = self.download_dir();
        let mut logger = paris::Logger::new();
        if !download_dir.exists() {
            logger.info(format!(
                "Creating download directory at {}",
                &download_dir.display()
            ));
            dir::create_all(&download_dir, false)?;
        }

        let source_url = Url::parse(&src)
            .wrap_err_with(|| format!("Unable to parse package source url {}", &src))?;

        if source_url.scheme().starts_with("http") {
            logger.info("Downloading remote package");
            Downloader::new(src, self).download()
        } else if source_url.scheme().starts_with("file") {
            let file_path = PathBuf::from(source_url.path());
            // Check if domain is available for case where file://./relative_file
            let path = match source_url.domain() {
                Some(domain) => {
                    let rel_path = fs::canonicalize(PathBuf::from(domain)).wrap_err_with(|| {
                        format!("Failed to get local absolute path for {}", domain)
                    })?;
                    rel_path.join(file_path.strip_prefix("/")?)
                }
                None => file_path,
            };

            if path.is_dir() {
                logger.info(format!("Installing from directory {}", path.display()));

                let manifest_path = path.join(Manifest::manifest_file_name());
                let manifest = Manifest::new_from_path(manifest_path)?;
                let download_package_dir = download_dir.join(manifest.id);
                logger.info(format!(
                    "Copying Package {} -> {}",
                    path.display(),
                    download_package_dir.display(),
                ));

                let mut options = dir::CopyOptions::new();
                options.copy_inside = true;
                options.overwrite = true;
                dir::copy(&path, &download_package_dir, &options).wrap_err_with(|| {
                    format!(
                        "Error copying package to {}",
                        download_package_dir.display()
                    )
                })?;

                Ok(DownloadedPackage {
                    local_path: download_package_dir,
                    package_service: self,
                })
            } else if path.is_file() {
                logger.info("Unpacking local file");
                let local_path = Packer::new(path).unpack(self.download_dir())?;
                Ok(DownloadedPackage {
                    local_path,
                    package_service: self,
                })
            } else {
                Err(eyre!("Path to package does not exist: {}", path.display()))
            }
        } else {
            Err(eyre!(
                "Package source URL has unrecognized scheme: {}",
                source_url.scheme()
            ))
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
                    "Removing package: {} @ {}",
                    installed_package.manifest.id, installed_package.manifest.version
                ));
                installed_package.remove().unwrap_or(false)
            })
            .filter(|was_removed| !was_removed)
            .collect::<Vec<bool>>();

        if errored_uninstalled.is_empty() {
            Ok(())
        } else {
            Err(eyre!("Failed to uninstall all packages"))
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

    pub fn persist_package_index(&self, package_index: PackageIndex) -> Result<usize> {
        let package_index_path = self.install_dir().join("index.toml");
        let index_toml = toml::to_string_pretty(&package_index)?;
        fs::write(package_index_path, &index_toml)?;

        Ok(index_toml.len())
    }

    pub fn load_package_index(&self) -> Result<PackageIndex> {
        let package_index_path = self.install_dir().join("index.toml");
        let index_toml = fs::read_to_string(package_index_path)?;
        let package_index: PackageIndex = toml::from_str(&index_toml)?;

        Ok(package_index)
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

        let src = String::from("file://./example-package");
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
    fn test_download() {
        defer!(purge());

        let src = String::from("file://./example-package");
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

        let src = String::from("file://./example-package");
        let package_service =
            PackageService::new().expect("Unable to instantiate PackageService in test");

        package_service
            .download_and_install(src)
            .expect("Unable to instantiate package in test");

        let test_package_service =
            PackageService::new().expect("Unable to instantiate PackageService in test");

        let installed_path = test_package_service
            .installed_package_path("org.hermione.example-package")
            .expect("Unable to remove example-package in test");
        assert!(installed_path.is_dir());
    }

    #[test]
    fn test_install_package_path() {
        defer!(purge());

        let package_name = "org.hermione.example-package";

        let package_service: PackageService =
            PackageService::new().expect("Could not create package service in test");

        package_service
            .download_and_install("file://./example-package".to_string())
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
