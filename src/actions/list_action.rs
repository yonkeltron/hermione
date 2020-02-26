use anyhow::Result;
use slog::info;

use crate::action::Action;
use crate::package_service::PackageService;

/// List Action displays a list of all currently installed Hermione Packages
pub struct ListAction {}

impl Action for ListAction {
    fn execute(self, package_service: PackageService) -> Result<()> {
        info!(package_service.logger, "Initialized");
        let installed_packages = package_service.list_installed_packages()?;
        installed_packages
            .iter()
            .enumerate()
            .for_each(|(index, installed_package)| {
                info!(
                    package_service.logger,
                    "{}. {}", (index + 1).to_string(), installed_package.package_name.clone();
                    "path" => installed_package.local_path.display(),
                    "package" => installed_package.package_name.clone(),
                )
            });
        info!(
            package_service.logger,
            "Displaying: {} Packages",
            installed_packages.len(),
        );
        Ok(())
    }
}
