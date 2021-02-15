use serde::{Deserialize, Serialize};

use std::collections::HashMap;

use crate::repositories::available_package::AvailablePackage;
use crate::repositories::package_index::PackageIndex;

#[derive(Deserialize, Debug, Serialize)]
pub struct RepositoryContents {
    pub name: String,
    pub available_packages: Vec<AvailablePackage>,
}

impl RepositoryContents {
    pub fn into_index(self) -> PackageIndex {
        let mut index = HashMap::new();

        for available_package in self.available_packages {
            index.insert(available_package.id, available_package.available_versions);
        }

        index
    }
}
