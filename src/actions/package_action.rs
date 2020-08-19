use anyhow::Result;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use paris::Logger;
use tar::Builder;

use std::fs;
use std::path::Path;

use crate::action::Action;
use crate::manifest::Manifest;
use crate::package_service::PackageService;

/// List Action displays a list of all currently installed Hermione Packages
pub struct PackageAction {
    pub package_path: String,
}

impl Action for PackageAction {
    fn execute(self, _package_service: PackageService) -> Result<()> {
        let mut logger = Logger::new();
        logger.info("Initialized");
        logger.loading("Loading package manifest");
        let package_path = Path::new(&self.package_path);
        let manifest_path = Path::new(&self.package_path).join("hermione.yml");
        let manifest = Manifest::new_from_path(manifest_path.to_path_buf())?;
        logger.info("Loaded package manifest.").newline(1);
        let archive_file = fs::File::create(format!("{}.hpkg", manifest.name))?;
        let encoder = ZlibEncoder::new(archive_file, Compression::default());
        let mut builder = Builder::new(encoder);

        logger.info(format!("Packaging {}", self.package_path));
        let mut mappings = Vec::new();
        for mut file_mapping_definition in manifest.mappings {
            file_mapping_definition.set_integrity(package_path.to_path_buf())?;
            let file_path = package_path.join(&file_mapping_definition.i);
            builder.append_path(&file_path)?;
            logger.indent(1).log(format!(
                "Added <blue>{}</> to package archive",
                file_mapping_definition.i
            ));
            mappings.push(file_mapping_definition);
        }
        let mut changed_manifest = Manifest::new_from_path(manifest_path.to_path_buf())?;
        changed_manifest.mappings = mappings;
        let manifest_yaml = serde_yaml::to_string(&changed_manifest)?;
        fs::write(&manifest_path, manifest_yaml)?;
        logger.info(format!(
            "Wrote integrity data to {}",
            manifest_path.display()
        ));
        builder.append_path(manifest_path)?;
        let finished_file_stream = builder.into_inner()?;
        finished_file_stream.finish()?;
        logger.success("Done.");

        Ok(())
    }
}
