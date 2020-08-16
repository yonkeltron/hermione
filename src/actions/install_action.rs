use anyhow::Result;
use paris::Logger;

use crate::action::Action;
use crate::package_service::PackageService;

/// Install Action installs a given Hermione package.
pub struct InstallAction {
    pub package_source: String,
}

impl Action for InstallAction {
    fn execute(self, package_service: PackageService) -> Result<()> {
        let mut logger = Logger::new();
        logger.info("Initialized").info(format!(
            "Downloading and installing from {}",
            &self.package_source
        ));
        package_service.download_and_install(self.package_source)?;
        logger.success("Done.");
        Ok(())
    }
}
