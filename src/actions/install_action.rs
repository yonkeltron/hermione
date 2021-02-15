use color_eyre::eyre::{eyre, Result};
use paris::Logger;

use crate::action::Action;
use crate::package_service::PackageService;

/// Install Action installs a given Hermione package.
pub struct InstallAction {
    pub package_source: String,
    pub package_version: Option<String>,
}

impl Action for InstallAction {
    fn execute(self, package_service: PackageService) -> Result<()> {
        let mut logger = Logger::new();

        let package_index = package_service.load_package_index()?;

        if package_index.contains_key(&self.package_source) {
            let available_versions = package_index
                .get(&self.package_source)
                .ok_or_else(|| eyre!("Package index has no ID {}", self.package_source))?;
            logger.info(format!("Fetching Package ID {}", self.package_source));
            let package_source = match self.package_version {
                Some(version) => available_versions
                    .iter()
                    .find(|available_version| available_version.version == version)
                    .ok_or_else(|| eyre!("Package has no {} version", version)),
                None => available_versions
                    .iter()
                    .max()
                    .ok_or_else(|| eyre!("Package has no latest version")),
            }?;

            logger.info("Initialized").info(format!(
                "Downloading and installing from {}",
                &package_source.url
            ));
            package_service.download_and_install(String::from(&package_source.url))?;
            logger.success("Done.");
            Ok(())
        } else {
            Err(eyre!(
                "Package ID {} not found in index, have you added a repo for it?",
                self.package_source
            ))
        }
    }
}
