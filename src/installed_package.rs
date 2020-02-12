use anyhow::Result;

use std::fs;
use std::path::PathBuf;

use crate::manifest::Manifest;
use crate::package_service::PackageService;

pub struct InstalledPackage {
    pub local_path: PathBuf,
    pub package_name: String,
    pub package_service: PackageService,
}

impl InstalledPackage {
    pub fn from_package_name(name: String) -> Result<Self> {
        let package_service = PackageService::new()?;
        let package_path = package_service.installed_package_path(&name)?;

        Ok(InstalledPackage {
            local_path: package_path,
            package_name: name,
            package_service,
        })
    }

    pub fn uninstall(&self) -> Result<bool> {
        let manifest_path = self.local_path.join("hermione.yml");
        let manifest = Manifest::new_from_path(manifest_path)?;

        for mapping_definition in manifest.mappings {
            let mapping = mapping_definition
                .render_file_mapping(&self.package_service, self.local_path.clone())?;
            println!("Successfully => {}", mapping.uninstall()?);
        }

        Ok(true)
    }

    pub fn remove(self) -> Result<bool> {
        self.uninstall()?;

        fs::remove_dir_all(&self.local_path)?;

        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use quickcheck_macros::quickcheck;

    #[quickcheck]
    fn from_package_name_with_bogus_package_always_fails(name: String) -> bool {
        InstalledPackage::from_package_name(name).is_err()
    }

    #[test]
    fn test_from_package_name_with_real_name() {
        let name = String::from("example-package");
        let package =
            PackageService::download(name.clone()).expect("Unable to install package in test");

        assert!(InstalledPackage::from_package_name(name).is_ok());

        fs::remove_dir_all(package.local_path).expect("Unable to remove package in test");
    }
}
