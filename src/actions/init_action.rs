use anyhow::Result;
use slog::info;

use std::path::Path;

use crate::action::Action;
use crate::package_service::PackageService;
use crate::scaffold::Scaffold;

pub struct InitAction {
    pub manifest_path: String,
}

impl Action for InitAction {
    fn execute(self, package_service: PackageService) -> Result<()> {
        info!(package_service.logger, "Initialized");
        let initialize_path = Path::new(&self.manifest_path);
        let scaffold = Scaffold::new("<Package Name>");
        scaffold.create_manifest(initialize_path.to_path_buf(), &package_service.logger)
    }
}
