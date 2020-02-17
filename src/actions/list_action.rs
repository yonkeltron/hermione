use anyhow::{anyhow, Result};

use crate::action::Action;
use crate::package_service::PackageService;

pub struct ListAction {
    pub yes_i_am_sure: bool,
}

impl Action for ListAction {
    fn execute(self, package_service: PackageService) -> Result<()> {
        Ok(())
    }
}
