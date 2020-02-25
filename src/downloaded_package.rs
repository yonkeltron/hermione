use anyhow::{anyhow, Result};
use fs_extra::dir;
use slog::{error, info};

use std::path::PathBuf;

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
            for mapping_result in mapping_render_results {
                match mapping_result {
                    Ok(mapping) => {
                        info!(self.package_service.logger, "{}", mapping.install()?);
                    }
                    Err(e) => {
                        error!(self.package_service.logger, "Failed to create");
                        eprintln!("Failed to resolve files destination {}", e.to_string())
                    }
                }
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
}
