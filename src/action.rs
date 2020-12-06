use color_eyre::eyre::Result;

use crate::package_service::PackageService;

/// CLI Action Trait
pub trait Action {
    fn execute(self, package_service: PackageService) -> Result<()>;
}
