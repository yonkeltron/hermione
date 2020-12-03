use color_eyre::eyre::{eyre, Result};
use paris::Logger;

use crate::action::Action;
use crate::package_service::PackageService;

/// Implode Action removes all installed packages and all downloaded packages,
/// and cleans up the install directory and the download directory.
pub struct ImplodeAction {
    pub yes_i_am_sure: bool,
}

impl Action for ImplodeAction {
    fn execute(self, package_service: PackageService) -> Result<()> {
        let mut logger = Logger::new();
        logger.info("Initialized");
        if self.yes_i_am_sure {
            package_service.implode()?;
            logger.success("Successfully removed everything Hermione");
            Ok(())
        } else {
            logger.error("I am not sure you want me to do this.");
            Err(eyre!("Please pass confirm flag if you are sure"))
        }
    }
}
