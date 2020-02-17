use anyhow::{anyhow, Result};

use crate::action::Action;
use crate::package_service::PackageService;

pub struct ImplodeAction {
    pub yes_i_am_sure: bool,
}

impl Action for ImplodeAction {
    fn execute(self, package_service: PackageService) -> Result<()> {
        if self.yes_i_am_sure {
            Ok(package_service.implode()?)
        } else {
            println!("I am not sure you want me to do this.");
            Err(anyhow!("Please pass confirm flag if you are sure"))
        }
    }
}
