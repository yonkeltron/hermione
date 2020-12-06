use color_eyre::eyre::{eyre, Result, WrapErr};
use fs_extra::dir;
use paris::Logger;

use std::fs;
use std::path::PathBuf;

use crate::downloader::Downloader;
use crate::file_mapping::FileMapping;
use crate::installed_package::InstalledPackage;
use crate::manifest::Manifest;
use crate::package_service::PackageService;

/// Downloaded state of a package.
/// This means that a package is downloaded in cache
/// but not currently installed.
pub struct DownloadedPackage {
    pub local_path: PathBuf,
    pub package_service: PackageService,
}

impl DownloadedPackage {
    /// Installs the downloaded package.
    ///
    /// Returns InstalledPackage Result.
    pub fn install(self) -> Result<InstalledPackage> {
        let mut logger = Logger::new();
        let manifest_path = self.local_path.join(Manifest::manifest_file_name());
        let manifest = Manifest::new_from_path(manifest_path)?;
        let package_id = manifest.id.clone();
        let mapping_render_results = manifest
            .mappings
            .clone()
            .into_iter()
            .filter(|mapping_definition| mapping_definition.valid_platform_family())
            .map(|mapping_definition| {
                let location = self
                    .package_service
                    .download_dir()
                    .join(&package_id.as_str());
                logger.info("Integrity Check").indent(1).log(format!(
                    "Input file: {}",
                    &location.join(&mapping_definition.i).display()
                ));
                match mapping_definition.verify_integrity(location) {
                    Ok(valid) => {
                        if valid {
                            mapping_definition.render_file_mapping(
                                &self.package_service,
                                self.package_service
                                    .install_dir()
                                    .join(&package_id.as_str()),
                            )
                        } else {
                            Err(eyre!("Integrity Check Failed!"))
                        }
                    }
                    Err(e) => Err(eyre!(
                        "Unable to conduct integrity check for {} | Reason: {}",
                        &mapping_definition.i,
                        e
                    )),
                }
            })
            .collect::<Vec<_>>();

        let mapping_render_errors = mapping_render_results
            .iter()
            .filter(|result| result.is_err())
            .collect::<Vec<_>>();
        if !mapping_render_errors.is_empty() {
            mapping_render_errors
                .into_iter()
                .for_each(|error| eprintln!("{:?}", error));
            Err(eyre!("Unable to install package"))
        } else {
            logger.info("Running preflight check");

            let validated_mappings = self
                .validate_mappings(mapping_render_results)
                .wrap_err_with(|| {
                    "Bailing on Install! Not all file mappings are valid.".to_string()
                })?;

            let mut copy_options = dir::CopyOptions::new();
            copy_options.copy_inside = true;
            copy_options.overwrite = true;
            let dest_path = self.package_service.install_dir();
            if !dest_path.exists() {
                logger.loading(format!(
                    "Creating install directory: {}",
                    &dest_path.display()
                ));
                dir::create_all(&dest_path, false)?;
                logger.info("Successfully created install directory");
            }
            logger.info("Installing");
            dir::copy(&self.local_path, &dest_path, &copy_options)?;
            let install_path = dest_path.join(&manifest.id);

            match &manifest.hooks {
                Some(hooks) => hooks.execute_pre_install()?,
                None => {
                    logger.log("No pre_install hook");
                }
            };
            logger.info("Linking files");
            for valid_mapping in validated_mappings {
                logger.indent(1).log(valid_mapping.install()?);
            }
            logger.success(format!("Successfully installed {}", &manifest.name));

            match &manifest.hooks {
                Some(hooks) => hooks.execute_post_install()?,
                None => {
                    logger.log("No post_install hook");
                }
            };

            Ok(InstalledPackage {
                local_path: install_path,
                manifest,
                package_service: self.package_service,
            })
        }
    }

    /// Remove the downloaded directory for the specified package.
    pub fn remove(&self) -> Result<()> {
        fs::remove_dir_all(&self.local_path)?;
        Ok(())
    }

    /// Upgrade the Downloaded package to the latest from the remote repo
    ///
    /// Returns a Result of InstalledPackage
    pub fn upgrade(self) -> Result<InstalledPackage> {
        let mut logger = Logger::new();
        let manifest_path = self.local_path.join(Manifest::manifest_file_name());
        let manifest = Manifest::new_from_path(manifest_path)?;
        logger.loading(format!("Started upgrading {}", &manifest.name));

        let downloader =
            Downloader::new(String::from("TODO Implement"), self.package_service.clone());
        match downloader.download() {
            Ok(_) => {
                logger.info("Finished fetching latest.");
                self.install()
            }
            Err(e) => {
                logger
                    .error("Could not upgrade package, reverting back")
                    .indent(1)
                    .log(format!("<red>{}</>", e.to_string()));
                self.install()
            }
        }
    }

    /// Checks that for a given vector of FileMapping results they all pass `pre_install_check()`
    /// Errors if any one of the file mappings fails the `pre_install_check()`.
    ///
    /// ### Arguments
    ///
    /// * mappings - A vector of Result FileMappings typically from collecting `FileMappingDefinition::render_file_mapping()`.
    ///
    /// Returns a Vector of FileMapping as a Result.
    fn validate_mappings(&self, mappings: Vec<Result<FileMapping>>) -> Result<Vec<FileMapping>> {
        let mut logger = Logger::new();
        logger.info("Validating mappings");
        mappings
            .into_iter()
            .map(|mapping_result| {
                let mapping = mapping_result?;
                logger
                    .indent(1)
                    .log(format!("OK: {}", mapping.pre_install_check()?));
                Ok(mapping)
            })
            .collect::<Result<Vec<_>>>()
    }
}
