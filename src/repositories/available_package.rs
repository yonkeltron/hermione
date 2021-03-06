use serde::{Deserialize, Serialize};

use crate::repositories::available_version::AvailableVersion;

#[derive(Deserialize, Serialize)]
pub struct AvailablePackage {
    pub available_versions: Vec<AvailableVersion>,
    pub homepage: String,
    pub id: String,
}
