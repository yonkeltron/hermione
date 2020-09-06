use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct AvailableVersion {
    pub version: String,
    pub url: String,
}
