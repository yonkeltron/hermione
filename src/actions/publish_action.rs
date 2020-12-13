use color_eyre::eyre::{eyre, Result, WrapErr};
use glob::glob;
use paris::Logger;
use url::Url;

use std::path::Path;

use crate::action::Action;
use crate::package_service::PackageService;
use crate::packer::Packer;
use crate::repositories::available_version::AvailableVersion;

pub struct PublishAction {
  pub url_prefix: String,
  pub repo_file: String,
  pub packages_dir: String,
}

impl Action for PublishAction {
  fn execute(self, package_service: PackageService) -> Result<()> {
    let url_prefix =
      Url::parse(&self.url_prefix).wrap_err("invalid or malformed prefix URL provided")?;
    let repo_file_path = Path::new(&self.repo_file);
    let packages_dir_path = Path::new(&self.packages_dir);

    if packages_dir_path.is_dir() {
      let mut logger = Logger::new();
      let glob_pattern = format!("{}/**/*.hpkg", packages_dir_path.display());
      let globs = glob(&glob_pattern)?;

      let available_versions = globs
        .into_iter()
        .filter(|entry| entry.is_ok())
        .map(|entry| entry.unwrap())
        .map(|package_path| {
          let manifest_result = Packer::new(package_path.clone()).get_manifest_from_archive();
          if manifest_result.is_ok() {
            let manifest = manifest_result.expect("unable to extract manifest");
            let package_name = package_path
              .file_name()
              .expect("unable to extract filename")
              .to_string_lossy();
            let version = manifest.version;
            match url_prefix.join(&package_name) {
              Ok(url) => {
                let available_version = AvailableVersion {
                  url: url.to_string(),
                  version,
                };

                Some(available_version)
              }
              Err(e) => {
                logger.warn(format!("Unable to construct URL: {}", e));

                None
              }
            }
          } else {
            logger.warn(format!(
              "Skipping invalid package {}",
              package_path.display()
            ));

            None
          }
        })
        .filter(|opt| opt.is_some())
        .collect::<Vec<_>>();

      Ok(())
    } else {
      Err(eyre!(
        "Unable to stat packages directory {}",
        packages_dir_path.display()
      ))
    }
  }
}
