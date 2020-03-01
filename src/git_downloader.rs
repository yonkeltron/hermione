use anyhow::{anyhow, Context, Result};
use git2::Repository;
use slog::info;

use std::fs;
use std::path::PathBuf;

use crate::downloaded_package::DownloadedPackage;
use crate::package_service::PackageService;

/// Represents the data required to download a repo from a git server.
pub struct GitDownloader {
    clone_path: PathBuf,
    package_name: String,
    package_service: PackageService,
}

impl GitDownloader {
    /// Returns an instance of GitDownloader.
    pub fn new(clone_path: PathBuf, package_name: String, package_service: PackageService) -> Self {
        Self {
            clone_path,
            package_name,
            package_service,
        }
    }

    /// Clones the git repo into a specified `Self.clone_path`, if a directory of
    /// the same package name already exists in the cache it is blown away and
    /// cloned afresh.
    pub fn download_or_update(self, src: String) -> Result<DownloadedPackage> {
        if self.clone_path.is_dir() {
            self.clone_fresh(src)
        } else {
            self.clone(src)
        }
    }

    /// Blow away the package path and clone it afresh from the remote `src`.
    fn clone_fresh(self, src: String) -> Result<DownloadedPackage> {
        info!(self.package_service.logger, "Obliterating cached package"; "path" => &self.clone_path.display());
        fs::remove_dir_all(&self.clone_path).with_context(|| {
            format!(
                "Failed to remove cache package path {}",
                self.clone_path.display()
            )
        })?;

        self.clone(src)
    }

    /// Execute a clone against the `src` and hydrate a `DownloadedPackage` if
    /// it succeeds.
    fn clone(self, src: String) -> Result<DownloadedPackage> {
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
