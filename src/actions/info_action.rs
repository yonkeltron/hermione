use anyhow::Result;
use slog::info;

use crate::action::Action;
use crate::package_service::PackageService;

/// Init Action creates a template `hermione.yml` file in the current directory it is ran.
pub struct InfoAction {
    pub package_name: String,
}

impl Action for InfoAction {
    fn execute(self, package_service: PackageService) -> Result<()> {
        let logger = package_service.logger.clone();
        info!(logger, "Initialized");
        let installed_package = package_service.get_installed_package(self.package_name)?;
        info!(logger, "{}", installed_package.describe()?);
        Ok(())
    }
}
