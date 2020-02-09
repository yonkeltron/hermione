use anyhow::Result;
use fs_extra::dir;

use std::path::PathBuf;

use crate::installed_package::InstalledPackage;
use crate::manifest::Manifest;
use crate::package_service::PackageService;

pub struct DownloadedPackage {
    pub local_path: PathBuf,
    pub package_name: String,
    pub package_service: PackageService,
}

impl DownloadedPackage {
    pub fn install(self) -> Result<InstalledPackage> {
        let manifest_path = self.local_path.join("hermione.yml");
        let manifest = Manifest::new_from_path(manifest_path)?;
        for mapping_definition in manifest.mappings {
            let mapping = mapping_definition.render_file_mapping()?;
            let activity_line = mapping.install(false)?;
            println!("{}", activity_line);
        }

        let copy_options = dir::CopyOptions::new();
        let dest_path = self.package_service.install_dir().join(&self.package_name);
        dir::copy(&self.local_path, dest_path, &copy_options)?;

        Ok(InstalledPackage {
            local_path: self.local_path,
            package_name: self.package_name,
            package_service: self.package_service,
        })
    }
}
