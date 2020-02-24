use anyhow::Result;
use slog::{debug, info};

use std::fs;
use std::path::PathBuf;

use crate::manifest::Manifest;
use crate::package_service::PackageService;

pub struct InstalledPackage {
    pub local_path: PathBuf,
    pub package_name: String,
    pub package_service: PackageService,
}

impl InstalledPackage {
    pub fn uninstall(&self) -> Result<bool> {
        let manifest_path = self.local_path.join("hermione.yml");
        info!(
            self.package_service.logger,
            "Removing files defined in Manifest file";
            "path" => manifest_path.display(),
        );

        let manifest = Manifest::new_from_path(manifest_path)?;
        debug!(self.package_service.logger, "{:#?}", &manifest);

        for mapping_definition in manifest.mappings {
            let mapping = mapping_definition
                .render_file_mapping(&self.package_service, self.local_path.clone())?;
            info!(self.package_service.logger, "{}", mapping.uninstall()?);
        }
        info!(
            self.package_service.logger,
            "Successfully removed files"; "package" => self.package_name.clone(),
        );
        Ok(true)
    }

    pub fn remove(self) -> Result<bool> {
        self.uninstall()?;

        fs::remove_dir_all(&self.local_path)?;

        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::logger::create_testing_logger;
    use scopeguard::defer;

    fn purge() {
        let package_service = PackageService::new(create_testing_logger())
            .expect("Unable to instantiate PackageService in test");
        package_service
            .implode()
            .expect("Failed to clean up in test");
    }

    #[test]
    fn test_from_package_name_with_real_name() {
        defer!(purge());
        let name = String::from("example-package");
        let package_service = PackageService::new(create_testing_logger())
            .expect("Unable to instantiate PackageService in test");
        let installed_package = package_service
            .download_and_install("./example-package".to_string())
            .expect("Failed to install package");

        let test_package_service = PackageService::new(create_testing_logger())
            .expect("Unable to instantiate PackageService in test");

        assert!(test_package_service.get_installed_package(name).is_ok());
        installed_package.remove().expect("Failed to clean up dir");
    }
}
