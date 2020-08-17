use anyhow::Result;
use paris::Logger;

use crate::action::Action;
use crate::package_service::PackageService;

/// Remove Action removes a currently installed Hermione Package.
pub struct RemoveAction {
    pub package_name: String,
}

impl Action for RemoveAction {
    fn execute(self, package_service: PackageService) -> Result<()> {
        let mut logger = Logger::new();
        logger.info("Initialized");
        let remove_result = match package_service.get_installed_package(self.package_name) {
            Ok(package) => package.remove(),
            Err(e) => Err(e),
        };

        match remove_result {
            Ok(_success) => logger.success("Removal successful"),
            Err(e) => logger.error(format!("Unable to remove because: {}", e.to_string())),
        };
        Ok(())
    }
}
