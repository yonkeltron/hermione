use anyhow::{anyhow, Result};
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use paris::Logger;
use tar::{Archive, Builder};

use std::fs;
use std::path::{Path, PathBuf};

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

    fn get_package_path_buf(&self) -> Result<PathBuf> {
        let mut logger = Logger::new();
        logger.info("Validating package path");
        let package_path = Path::new(&self.package_string_path);
        if package_path.is_dir() {
            logger.indent(1).log("Path ok");
            Ok(package_path.to_path_buf())
        } else {
            Err(anyhow!(
                "Package path ({}) is not a directory",
                package_path.display()
            ))
        }
    }

    fn get_manifest_path_buf(&self, package_path: &PathBuf) -> Result<PathBuf> {
        let mut logger = Logger::new();
        logger.info("Validating manifest path");

        let manifest_path = package_path.join("hermione.yml");
        if manifest_path.exists() {
            logger.indent(1).log("Path ok");
            Ok(manifest_path)
        } else {
            logger.error("Path error with manifest");
            Err(anyhow!("Path error in manifest"))
        }
    }

    pub fn pack(self) -> Result<String> {
        let mut logger = Logger::new();
        let package_path = self.get_package_path_buf()?;
        let manifest_path = self.get_manifest_path_buf(&package_path)?;

        logger.loading("Loading package manifest");
        let manifest = Manifest::new_from_path(manifest_path.to_path_buf())?;
        logger.info("Loaded package manifest.").newline(1);

        let archive_file_location = format!("{}.hpkg", manifest.name);
        let archive_file = fs::File::create(&archive_file_location)?;
        let encoder = GzEncoder::new(archive_file, Compression::best());
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

    pub fn unpack(self, dest: PathBuf) -> Result<String> {
        let mut logger = Logger::new();
        logger.loading(format!(
            "Starting unpack on file {}",
            &self.package_string_path
        ));
        let archive_file = fs::File::open(&self.package_string_path)?;
        let decoder = GzDecoder::new(archive_file);
        let mut archive = Archive::new(decoder);

        archive.unpack(&dest)?;
        logger.success("Done");
        Ok(format!("{}", dest.display()))
    }
}
