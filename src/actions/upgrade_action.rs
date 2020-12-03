use color_eyre::eyre::Result;
use paris::Logger;

use crate::action::Action;
use crate::package_service::PackageService;

/// Upgrade Action upgrades a package
pub struct UpgradeAction {
    pub package_names: Vec<String>,
}

impl Action for UpgradeAction {
    fn execute(self, package_service: PackageService) -> Result<()> {
        let mut logger = Logger::new();
        let packages_to_upgrade = if self.package_names.is_empty() {
            logger.info("No packages given, defaulting to all of them");
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
            let downloaded_package = installed_package.uninstall()?;
            downloaded_package.upgrade()?;
        }

        Ok(())
    }
}
