use std::collections::HashMap;

use crate::repositories::available_version::AvailableVersion;

pub type PackageIndex = HashMap<String, Vec<AvailableVersion>>;
