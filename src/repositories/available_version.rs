use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Serialize)]
pub struct AvailableVersion {
    pub version: String,
    pub url: String,
}
