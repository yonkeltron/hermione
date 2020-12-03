use color_eyre::eyre::Result;
use paris::Logger;

use crate::action::Action;
use crate::package_service::PackageService;
use crate::scaffold::Scaffold;

/// New Action scaffolds a Hermione package directory with a `hermione.yml` file and a couple of sample files.
pub struct NewAction {
    pub package_name: String,
    pub package_id: String,
}

impl Action for NewAction {
    fn execute(self, _package_service: PackageService) -> Result<()> {
        let mut logger = Logger::new();
        logger.info("Initialized");
        let scaffold = Scaffold::new(&self.package_name, &self.package_id);
        scaffold.create_package()
    }
}
