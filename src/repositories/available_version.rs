use semver::Version;
use serde::{Deserialize, Serialize};

use std::cmp::Ordering;

#[derive(Deserialize, Debug, Serialize, PartialEq, Eq)]
pub struct AvailableVersion {
    pub version: String,
    pub url: String,
}

impl PartialOrd for AvailableVersion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let semver_version_opt = Version::parse(&self.version).ok();
        let other_semver_version_opt = Version::parse(&other.version).ok();

        if let Some(semver_version) = semver_version_opt {
            if let Some(other_semver_version) = other_semver_version_opt {
                Some(semver_version.cmp(&other_semver_version))
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl Ord for AvailableVersion {
    fn cmp(&self, other: &Self) -> Ordering {
        let semver_version_opt = Version::parse(&self.version).ok();
        let other_semver_version_opt = Version::parse(&other.version).ok();

        if let Some(semver_version) = semver_version_opt {
            if let Some(other_semver_version) = other_semver_version_opt {
                semver_version.cmp(&other_semver_version)
            } else {
                Ordering::Equal
            }
        } else {
            Ordering::Equal
        }
    }
}
