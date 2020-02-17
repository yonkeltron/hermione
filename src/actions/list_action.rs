use anyhow::Result;

use crate::action::Action;
use crate::package_service::PackageService;

pub struct ListAction {}

impl Action for ListAction {
    fn execute(self, package_service: PackageService) -> Result<()> {
        let installed_packages = package_service.list_installed_packages()?;
        println!("Displaying: {} Packages", installed_packages.len());
        installed_packages
            .iter()
            .for_each(|installed_package| println!("{}", installed_package.package_name));
        Ok(())
    }
}
