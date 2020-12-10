use color_eyre::eyre::{eyre, Result, WrapErr};
use glob::glob;
use paris::Logger;
use url::Url;

use std::path::Path;

use crate::action::Action;
use crate::package_service::PackageService;
use crate::packer::Packer;

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

      globs
        .into_iter()
        .filter(|entry| entry.is_ok())
        .map(|entry| entry.unwrap())
        .map(|package_path| {
          let manifest_result = Packer::new(package_path).get_manifest_from_archive();
          if manifest_result.is_ok() {
            let manifest = manifest_result.expect("unable to extract manifest");
            Some(manifest)
          } else {
            logger.warn(format!(
              "Skipping invalid package {}",
              package_path.display()
            ));

            None
          }
        })
        .filter(|manifest_option| manifest_option.is_some())
        .map(|manifest_option| manifest_option.unwrap());

      Ok(())
    } else {
      Err(eyre!(
        "Unable to stat packages directory {}",
        packages_dir_path.display()
      ))
    }
  }
}
