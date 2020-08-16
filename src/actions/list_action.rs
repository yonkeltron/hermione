use anyhow::Result;
use paris::Logger;

use crate::action::Action;
use crate::package_service::PackageService;

/// List Action displays a list of all currently installed Hermione Packages
pub struct ListAction {}

impl Action for ListAction {
    fn execute(self, package_service: PackageService) -> Result<()> {
        let mut logger = Logger::new();
        logger.info("Initialized");
        let installed_packages = package_service.list_installed_packages()?;
        installed_packages
            .iter()
            .enumerate()
            .for_each(|(index, installed_package)| {
                logger.indent(1).info(format!(
                    "{}. {}",
                    (index + 1).to_string(),
                    installed_package.package_name.clone()
                ));
            });
        logger.success(format!("Displayed: {} Packages", installed_packages.len(),));
        Ok(())
    }
}
