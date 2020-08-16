use anyhow::{anyhow, Context, Result};
use git2::{build, Repository};
use paris::Logger;

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
        if self.clone_path.exists() {
            self.clone_fresh(src)
        } else {
            self.clone(src)
        }
    }

    /// Update does a git fetch on the latest master from remote origin
    pub fn update(self) -> Result<()> {
        self.update_branch("origin", "master")
    }

    /// Update Branch does a git fetch on a given remote and branch
    pub fn update_branch(self, remote: &str, branch: &str) -> Result<()> {
        let repo = Repository::open(&self.clone_path)?;
        let mut remote = repo
            .find_remote(remote)
            .or_else(|_| repo.remote_anonymous(remote))
            .with_context(|| {
                format!(
                    "Unable to set a remote called {} for the git repo at {}",
                    remote,
                    self.clone_path.display()
                )
            })?;

        remote
            .fetch(&[branch], None, None)
            .with_context(|| format!("Unable to fetch '{}'", branch))?;
        let fetch_head = repo.find_reference("FETCH_HEAD")?;
        let fetch_commit = repo.reference_to_annotated_commit(&fetch_head)?;
        let analysis = repo.merge_analysis(&[&fetch_commit])?;
        let mut logger = Logger::new();

        if analysis.0.is_fast_forward() {
            let ref_name = format!("refs/heads/{}", branch);
            logger.info(format!("Pulling latest {}", &ref_name));
            match repo.find_reference(&ref_name) {
                Ok(mut r) => {
                    let name = match r.name() {
                        Some(s) => s.to_string(),
                        None => String::from_utf8_lossy(r.name_bytes()).to_string(),
                    };
                    let ref_id = fetch_commit.id();
                    let msg = format!("Fast-Forward: Setting {} to id: {}", name, ref_id);
                    logger.info(&msg);
                    r.set_target(ref_id, &msg)?;
                    repo.set_head(&name)?;
                    repo.checkout_head(Some(build::CheckoutBuilder::default().force()))?;
                    Ok(())
                }
                Err(_) => {
                    logger.error("Could not pull latest from repo");
                    Err(anyhow!("Could not pulled latest repo info"))
                }
            }
        } else {
            logger.info("Package already up to date");
            Ok(())
        }
    }

    /// Blow away the package path and clone it afresh from the remote `src`.
    fn clone_fresh(self, src: String) -> Result<DownloadedPackage> {
        let mut logger = Logger::new();
        logger.info(format!(
            "Obliterating cached package {}",
            &self.clone_path.display()
        ));
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
        let mut logger = Logger::new();
        logger.info(format!("Cloning remote package from source: {}", &src));
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
