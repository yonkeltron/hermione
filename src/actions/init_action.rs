use anyhow::Result;
use slog::info;

use std::path::Path;

use crate::action::Action;
use crate::package_service::PackageService;
use crate::scaffold::Scaffold;

/// Init Action creates a template `hermione.yml` file in the current directory it is ran.
pub struct InitAction {}

impl Action for InitAction {
    fn execute(self, package_service: PackageService) -> Result<()> {
        info!(package_service.logger, "Initialized");
        let initialize_path = Path::new(".");
        let scaffold = Scaffold::new("<Package Name>");
        scaffold.create_manifest(initialize_path.to_path_buf(), &package_service.logger)
    }
}
