use anyhow::Result;
use slog::info;

use crate::action::Action;
use crate::package_service::PackageService;

pub struct InstallAction {
    pub package_source: String,
}

impl Action for InstallAction {
    fn execute(self, package_service: PackageService) -> Result<()> {
        info!(package_service.logger, "Initialized"; "package" => self.package_source.clone());
        info!(
            package_service.logger,
            "Downloading and installing"; "source" => &self.package_source,
        );
        package_service.download_and_install(self.package_source)?;
        Ok(())
    }
}
