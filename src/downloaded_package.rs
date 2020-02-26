use anyhow::{anyhow, Context, Result};
use fs_extra::dir;
use slog::{error, info};

use std::path::PathBuf;

use crate::file_mapping::FileMapping;
use crate::installed_package::InstalledPackage;
use crate::manifest::Manifest;
use crate::package_service::PackageService;

/// Downloaded state of a package.
/// This means that a package is downloaded in cache
/// but not currently installed.
pub struct DownloadedPackage {
    pub local_path: PathBuf,
    pub package_name: String,
    pub package_service: PackageService,
}

impl DownloadedPackage {
    /// Installs the downloaded package.
    ///
    /// Returns InstalledPackage Result.
    pub fn install(self) -> Result<InstalledPackage> {
        let manifest_path = self.local_path.join("hermione.yml");
        let manifest = Manifest::new_from_path(manifest_path)?;
        let mapping_render_results = manifest
            .mappings
            .into_iter()
            .map(|mapping_definition| {
                mapping_definition
                    .render_file_mapping(&self.package_service, self.local_path.clone())
            })
            .collect::<Vec<_>>();

        let mapping_render_errors = mapping_render_results
            .iter()
            .filter(|result| result.is_err())
            .collect::<Vec<_>>();
        if !mapping_render_errors.is_empty() {
            mapping_render_errors
                .iter()
                .for_each(|error| eprintln!("{:?}", error));
            Err(anyhow!("Unable to install package"))
        } else {
            info!(self.package_service.logger,
                "Validating file mappings before install";
                "package" => self.package_name.clone());

            let validated_mappings = self
                .validate_mappings(mapping_render_results)
                .with_context(|| format!("Bailing on Install! Not all file mappings are valid."))?;

            for valid_mapping in validated_mappings {
                info!(self.package_service.logger, "{}", valid_mapping.install()?);
            }
            let mut copy_options = dir::CopyOptions::new();
            copy_options.copy_inside = true;
            let dest_path = self.package_service.install_dir();
            if !dest_path.exists() {
                info!(
                    self.package_service.logger,
                    "Creating install directory";
                    "path" => &dest_path.display()
                );
                dir::create_all(&dest_path, false)?;
                info!(
                    self.package_service.logger,
                    "Successfully created install directory"
                );
            }
            info!(
                self.package_service.logger,
                "Installing"; "path" => &self.package_name,
            );
            dir::copy(&self.local_path, &dest_path, &copy_options)?;
            let install_path = dest_path.join(&self.package_name);

            info!(self.package_service.logger, "Successfully installed"; 
            "path" => install_path.display(),
            "package" => self.package_name.clone());

            Ok(InstalledPackage {
                local_path: install_path,
                package_name: self.package_name,
                package_service: self.package_service,
            })
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
        let mut validated_mappings: Vec<FileMapping> = vec![];
        for mapping_result in mappings {
            match mapping_result {
                Ok(mapping) => {
                    info!(
                        self.package_service.logger,
                        " + {}",
                        mapping.pre_install_check()?;
                        "package" => self.package_name.clone(),
                    );
                    validated_mappings.push(mapping);
                }
                Err(e) => {
                    error!(self.package_service.logger, "Failed to create");
                    eprintln!("Failed to resolve files destination {}", e.to_string())
                }
            }
        }
        Ok(validated_mappings)
    }
}
