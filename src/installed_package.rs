use color_eyre::eyre::Result;
use paris::Logger;

use std::fs;
use std::path::PathBuf;

use crate::downloaded_package::DownloadedPackage;
use crate::manifest::Manifest;
use crate::package_service::PackageService;

/// Installed state of a package.
/// This means that a package has been downloaded and the
/// files in the manifest have been installed in their corresponding spot.
pub struct InstalledPackage {
    /// Local PathBuf of the installed package.
    pub local_path: PathBuf,
    /// Manifest.
    pub manifest: Manifest,
    /// Instance of PackageService.
    pub package_service: PackageService,
}

impl InstalledPackage {
    /// Removes a given package.
    /// First we try to remove all the files it installed in the
    /// manifest before we remove the package directory itself.
    pub fn uninstall(&self) -> Result<DownloadedPackage> {
        let manifest_path = self.local_path.join(Manifest::manifest_file_name());
        let mut logger = Logger::new();
        logger.info(format!(
            "Unlinking files defined in Manifest file: {}",
            manifest_path.display(),
        ));

        let manifest = Manifest::new_from_path(&manifest_path)?;

        for mapping_definition in manifest.mappings {
            if mapping_definition.valid_platform_family() {
                let mapping = mapping_definition
                    .render_file_mapping(&self.package_service, self.local_path.clone())?;
                logger.indent(1).log(mapping.uninstall()?);
            }
        }
        logger.success("Successfully unlinked files");

        fs::remove_dir_all(&self.local_path)?;
        logger.success(format!(
            "Successfully removed installed package {}",
            &manifest.name,
        ));

        let downloaded_path_buf = self.package_service.download_dir().join(&manifest.id);

        Ok(DownloadedPackage {
            local_path: downloaded_path_buf,
            package_service: self.package_service.clone(),
        })
    }

    /// Removed the package directory it self after the files of this
    /// package have been successfully uninstalled.
    pub fn remove(self) -> Result<bool> {
        let manifest_path = self.local_path.join(Manifest::manifest_file_name());
        let manifest = Manifest::new_from_path(&manifest_path)?;

        let downloaded_package = self.uninstall()?;
        let mut logger = Logger::new();
        match &manifest.hooks {
            Some(hooks) => hooks.execute_pre_remove()?,
            None => {
                logger.log("No pre_remove hook");
            }
        };

        downloaded_package.remove()?;

        match manifest.hooks {
            Some(hooks) => hooks.execute_post_remove()?,
            None => {
                logger.log("No post_remove hook");
            }
        };

        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use scopeguard::defer;

    fn purge() {
        let package_service =
            PackageService::new().expect("Unable to instantiate PackageService in test");
        package_service
            .implode()
            .expect("Failed to clean up in test");
    }

    #[test]
    fn test_from_package_name_with_real_name() {
        defer!(purge());
        let name = String::from("org.hermione.example-package");
        let package_service =
            PackageService::new().expect("Unable to instantiate PackageService in test");
        let installed_package = package_service
            .download_and_install("file://./example-package".to_string())
            .expect("Failed to install package");

        let test_package_service =
            PackageService::new().expect("Unable to instantiate PackageService in test");

        assert!(test_package_service.get_installed_package(name).is_ok());
        installed_package.remove().expect("Failed to clean up dir");
    }
}
