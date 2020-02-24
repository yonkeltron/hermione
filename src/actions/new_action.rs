use anyhow::Result;
use slog::info;

use crate::action::Action;
use crate::package_service::PackageService;
use crate::scaffold::Scaffold;

pub struct NewAction {
    pub package_name: String,
}

impl Action for NewAction {
    fn execute(self, package_service: PackageService) -> Result<()> {
        info!(package_service.logger, "Initialized");
        let scaffold = Scaffold::new(&self.package_name);
        scaffold.create_package(&package_service.logger)
    }
}
