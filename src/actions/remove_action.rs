use anyhow::Result;
use slog::{error, info, o};

use crate::action::Action;
use crate::package_service::PackageService;

/// Remove Action removes a currently installed Hermione Package.
pub struct RemoveAction {
    pub package_name: String,
}

impl Action for RemoveAction {
    fn execute(self, package_service: PackageService) -> Result<()> {
        let logger = package_service
            .logger
            .new(o!("package" => self.package_name.clone()));
        info!(logger, "Initialized");
        let remove_result = match package_service.get_installed_package(self.package_name) {
            Ok(package) => package.remove(),
            Err(e) => Err(e),
        };

        match remove_result {
            Ok(_success) => info!(logger, "Removal successful"),
            Err(e) => error!(
                logger,
                "Unable to remove";
                "error" => e.to_string()
            ),
        };
        Ok(())
    }
}
