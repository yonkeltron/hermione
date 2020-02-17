use anyhow::Result;

use crate::action::Action;
use crate::package_service::PackageService;

pub struct RemoveAction {
    pub package_name: String,
}

impl Action for RemoveAction {
    fn execute(self, package_service: PackageService) -> Result<()> {
        let remove_result = match package_service.get_installed_package(self.package_name.clone()) {
            Ok(package) => package.remove(),
            Err(e) => Err(e),
        };

        match remove_result {
            Ok(_success) => println!("Removed package {}", self.package_name),
            Err(e) => eprintln!(
                "Unable to remove {} because {}",
                self.package_name,
                e.to_string()
            ),
        };
        Ok(())
    }
}
