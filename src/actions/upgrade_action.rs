use anyhow::{anyhow, Context, Result};
use git2::Repository;
use slog::{error, info};

use crate::action::Action;
use crate::package_service::PackageService;

/// Implode Action removes all installed packages and all downloaded packages,
/// and cleans up the install directory and the download directory.
pub struct UpgradeAction {
    pub package_names: Vec<String>,
}

impl Action for UpgradeAction {
    fn execute(self, package_service: PackageService) -> Result<()> {
        let packages_to_upgrade = if self.package_names.is_empty() {
            info!(
                package_service.logger,
                "No packages given, defaulting to all of them"
            );
            package_service.list_installed_packages()?
        } else {
            self.package_names
                .iter()
                .map(|package_name| {
                    package_service
                        .clone()
                        .get_installed_package(String::from(package_name))
                })
                .filter(|res| res.is_ok())
                .map(|res| res.expect("Unable to unwrap an InstalledPackage"))
                .collect()
        };

        for installed_package in packages_to_upgrade {
            installed_package.uninstall()?;
            let repo = Repository::open(&installed_package.local_path).with_context(|| {
                format!(
                    "Can't open the repo at {}",
                    installed_package.local_path.display()
                )
            })?;
            let mut remote = repo
                .find_remote("origin")
                .or_else(|_| repo.remote_anonymous("origin"))
                .with_context(|| {
                    format!(
                        "Unable to set a remote called 'origin' for the git repo at {}",
                        installed_package.local_path.display()
                    )
                })?;
            remote
                .fetch(&["master"], None, None)
                .with_context(|| format!("Unable to fetch 'master'"))?;
            //installed_package.relink()?;
        }

        Ok(())
    }
}
