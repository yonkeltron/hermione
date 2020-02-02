use anyhow::{anyhow, Result};

use std::fs;
use std::path::PathBuf;

use crate::manifest::Manifest;
use crate::package::Package;

pub struct InstalledPackage {
    pub local_path: PathBuf,
}

impl InstalledPackage {
    pub fn from_package_name(hermione_home: String, name: String) -> Result<Self> {
        let package_path = Package::install_path(&hermione_home, &name);

        if package_path.is_dir() {
            Ok(InstalledPackage {
                local_path: package_path,
            })
        } else {
            Err(anyhow!("It appears that {} isn't installed", name))
        }
    }

    pub fn uninstall(&self) -> Result<bool> {
        let manifest_path = self.local_path.join("hermione.yml");
        let manifest_path_string = format!("{}", manifest_path.display());
        let manifest = Manifest::new_from_file(&manifest_path_string)?;

        for mapping in manifest.mappings {
            mapping.uninstall()?;
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

    use crate::config::Config;

    #[quickcheck]
    fn from_package_name_with_bogus_package_always_fails(home: String, name: String) -> bool {
        InstalledPackage::from_package_name(home, name).is_err()
    }

    #[test]
    fn test_from_package_name_with_real_name() {
        let config = Config::load().expect("Unable to load config in test");
        config
            .init_hermione_home()
            .expect("Unable to init Hermione home in test");

        let home = config.hermione_home.clone();
        let name = String::from("example-package");
        let package =
            Package::download(name.clone(), &config).expect("Unable to install package in test");

        assert!(InstalledPackage::from_package_name(home, name).is_ok());

        package.remove().expect("Unable to remove package in test");
    }
}
