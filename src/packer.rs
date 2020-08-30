use anyhow::{anyhow, Result};
use flate2::write::GzEncoder;
use flate2::Compression;
use paris::Logger;
use tar::Builder;

use std::fs;
use std::path::Path;

use crate::manifest::Manifest;

pub struct Packer {
    pub package_string_path: String,
}

impl Packer {
    pub fn new(package_string_path: String) -> Self {
        Self {
            package_string_path,
        }
    }

    pub fn pack(self) -> Result<String> {
        let mut logger = Logger::new();
        logger.info("Validating package path");
        let package_path = Path::new(&self.package_string_path);
        match package_path.metadata() {
            Ok(stat) => {
                if !stat.is_dir() {
                    Err(anyhow!(
                        "Package path ({}) is not a directory",
                        package_path.display()
                    ))
                } else {
                    logger.indent(1).log("Path ok");
                    Ok(())
                }
            }
            Err(e) => {
                logger.error("Path error with package");
                Err(anyhow!(e))
            }
        }?;

        logger.info("Validating manifest path");
        let manifest_path = package_path.join("hermione.yml");
        match manifest_path.metadata() {
            Ok(_) => {
                logger.indent(1).log("Path ok");
                Ok(())
            }
            Err(e) => {
                logger.error("Path error with manifest");
                Err(anyhow!(e))
            }
        }?;

        logger.loading("Loading package manifest");
        let manifest = Manifest::new_from_path(manifest_path.to_path_buf())?;
        logger.info("Loaded package manifest.").newline(1);

        let archive_file_location = format!("{}.hpkg", manifest.name);
        let archive_file = fs::File::create(&archive_file_location)?;
        let encoder = GzEncoder::new(archive_file, Compression::default());
        let mut builder = Builder::new(encoder);

        logger.info(format!("Packaging {}", package_path.display()));
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
        builder.finish()?;
        builder.into_inner()?;
        logger.success("Done.");

        let loc = fs::canonicalize(Path::new(&archive_file_location))?;
        Ok(format!("{}", loc.to_string_lossy()))
    }

    pub fn unpack(self) -> Result<String> {
        let mut logger = Logger::new();
        logger.loading("Starting unpack");
        logger.success("Done");
        Ok(format!("Todo"))
    }
}
