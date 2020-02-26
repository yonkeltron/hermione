use anyhow::{anyhow, Result};
use git2::Repository;
use slog::info;

use std::fs;
use std::path::PathBuf;

use crate::downloaded_package::DownloadedPackage;
use crate::package_service::PackageService;

pub struct GitDownloader {
    clone_path: PathBuf,
    package_name: String,
    package_service: PackageService,
}

impl GitDownloader {
    pub fn new(clone_path: PathBuf, package_name: String, package_service: PackageService) -> Self {
        Self {
            clone_path,
            package_name,
            package_service,
        }
    }

    pub fn download_or_update(self, src: String) -> Result<DownloadedPackage> {
        if self.clone_path.is_dir() {
            info!(self.package_service.logger, "Obliterating cached package"; "path" => &self.clone_path.display());
            fs::remove_dir_all(&self.clone_path)?; // FIXME add context
        }

        info!(self.package_service.logger, "Cloning remote package"; "source" => &src);
        match Repository::clone(&src, &self.clone_path) {
            Ok(_repo) => Ok(DownloadedPackage {
                local_path: self.clone_path,
                package_name: self.package_name,
                package_service: self.package_service,
            }),
            Err(e) => Err(anyhow!(
                "Unable to git clone package from {} because {}",
                src,
                e
            )),
        }
    }
}
