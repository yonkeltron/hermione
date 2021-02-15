use color_eyre::eyre::Result;
use paris::Logger;

use crate::action::Action;
use crate::package_service::PackageService;

/// List Action displays a list of all currently installed Hermione Packages
pub struct ListAction {
    pub list_available: bool,
}

impl Action for ListAction {
    fn execute(self, package_service: PackageService) -> Result<()> {
        let mut logger = Logger::new();
        logger.info("Initialized");
        if self.list_available {
            logger.info("Available Packages:");
            package_service
                .load_package_index()?
                .into_iter()
                .for_each(|(package_id, versions)| {
                    logger.indent(1).log(format!(
                        "Package ID: {} ({} versions)",
                        package_id,
                        versions.len()
                    ));
                });
            Ok(())
        } else {
            let installed_packages = package_service.list_installed_packages()?;
            installed_packages
                .iter()
                .enumerate()
                .for_each(|(index, installed_package)| {
                    logger.indent(1).info(format!(
                        "{}. {} @ {}",
                        (index + 1).to_string(),
                        installed_package.manifest.id,
                        installed_package.manifest.version
                    ));
                });
            logger.success(format!("Displayed: {} Packages", installed_packages.len()));
            Ok(())
        }
    }
}
