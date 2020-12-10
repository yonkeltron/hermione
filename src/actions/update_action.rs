use color_eyre::eyre::Result;
use paris::Logger;

use crate::action::Action;
use crate::config::HermioneConfig;
use crate::package_service::PackageService;

/// Upgrade Action upgrades a package
pub struct UpdateAction;

impl Action for UpdateAction {
  fn execute(self, package_service: PackageService) -> Result<()> {
    let mut logger = Logger::new();

    logger.loading("Loading config...");
    let hermione_config = HermioneConfig::load()?;
    logger.log("Loaded config.");

    let package_index = hermione_config.fetch_and_build_index()?;

    logger.loading("Persisting new package index...");
    let toml_byte_count = package_service.persist_package_index(package_index)?;
    logger.info(format!("Wrote {} bytes to package index.", toml_byte_count));

    Ok(())
  }
}
