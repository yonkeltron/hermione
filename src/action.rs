use anyhow::Result;

use crate::package_service::PackageService;

pub trait Action {
    fn execute(self, package_service: PackageService) -> Result<()>;
}
