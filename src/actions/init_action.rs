use color_eyre::eyre::Result;
use paris::Logger;

use std::path::Path;

use crate::action::Action;
use crate::package_service::PackageService;
use crate::scaffold::Scaffold;

/// Init Action creates a template `hermione.yml` file in the current directory it is ran.
pub struct InitAction {}

impl Action for InitAction {
    fn execute(self, _package_service: PackageService) -> Result<()> {
        let mut logger = Logger::new();
        logger.info("Initialized");
        let initialize_path = Path::new(".");
        let scaffold = Scaffold::new("<Package Name>", "com.example.package");
        scaffold.create_manifest(initialize_path.to_path_buf())
    }
}
