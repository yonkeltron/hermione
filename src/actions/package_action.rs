use anyhow::Result;
use paris::Logger;

use crate::action::Action;
use crate::package_service::PackageService;
use crate::packer::Packer;

/// List Action displays a list of all currently installed Hermione Packages
pub struct PackageAction {
    pub package_path: String,
}

impl Action for PackageAction {
    fn execute(self, _package_service: PackageService) -> Result<()> {
        let mut logger = Logger::new();
        logger.info("Initialized");
        match Packer::new(self.package_path).pack() {
            Ok(archive_location) => {
                logger.info(format!("Archive Created at Path: {}", archive_location));
                Ok(())
            }
            Err(e) => Err(e),
        }
    }
}
