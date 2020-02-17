use anyhow::Result;

use crate::action::Action;
use crate::package_service::PackageService;

pub struct InstallAction {
    pub package_source: String,
}

impl Action for InstallAction {
    fn execute(self, package_service: PackageService) -> Result<()> {
        package_service.download_and_install(self.package_source)?;
        Ok(())
    }
}
