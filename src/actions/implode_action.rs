use anyhow::{anyhow, Result};
use slog::{error, info};

use crate::action::Action;
use crate::package_service::PackageService;

pub struct ImplodeAction {
    pub yes_i_am_sure: bool,
}

impl Action for ImplodeAction {
    fn execute(self, package_service: PackageService) -> Result<()> {
        info!(package_service.logger, "Initialized");
        if self.yes_i_am_sure {
            package_service.implode()?;
            info!(
                package_service.logger,
                "Successfully removed everything Hermione"
            );
            Ok(())
        } else {
            error!(
                package_service.logger,
                "I am not sure you want me to do this."
            );
            Err(anyhow!("Please pass confirm flag if you are sure"))
        }
    }
}
