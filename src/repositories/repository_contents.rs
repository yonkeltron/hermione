use serde::{Deserialize, Serialize};

use crate::repositories::available_package::AvailablePackage;

#[derive(Deserialize, Serialize)]
pub struct RepositoryContents {
  pub name: String,
  pub url: String,
  pub available_packages: Vec<AvailablePackage>,
}
