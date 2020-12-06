use serde::{Deserialize, Serialize};

use std::collections::HashMap;

use crate::repositories::available_package::AvailablePackage;
use crate::repositories::available_version::AvailableVersion;

#[derive(Deserialize, Serialize)]
pub struct RepositoryContents {
    pub name: String,
    pub available_packages: Vec<AvailablePackage>,
}

impl RepositoryContents {
    pub fn to_index(self) -> HashMap<String, Vec<AvailableVersion>> {
        let mut index = HashMap::new();

        for available_package in self.available_packages {
            index.insert(available_package.id, available_package.available_versions);
        }

        index
    }
}
