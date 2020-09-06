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
    pub package_path_buf: PathBuf,
}

impl Packer {
    pub fn new(package_path_buf: PathBuf) -> Self {
        Self { package_path_buf }
    }

    fn get_package_path_buf(&self) -> Result<PathBuf> {
        let mut logger = Logger::new();
        logger.info("Validating package path");
        let package_path = &self.package_path_buf;
        if package_path.is_dir() {
            logger
                .indent(1)
                .log(format!("Path Ok | {}", package_path.display()));
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
            logger
                .indent(1)
                .log(format!("Path Ok | {}", manifest_path.display()));
            Ok(manifest_path)
        } else {
            logger.error("Path error with manifest");
            Err(anyhow!("Path error in manifest"))
        }
    }

    fn get_manifest_from_archive(&self) -> Result<Manifest> {
        let archive_file = fs::File::open(&self.package_path_buf)?;
        let decoder = GzDecoder::new(archive_file);
        let mut archive = Archive::new(decoder);
        if let Some(file_path) = archive
            .entries()?
            .filter_map(|entry| entry.ok())
            .find(|entry| match entry.path() {
                Ok(path) => {
                    if let Some(file_name) = path.file_name() {
                        file_name == "hermione.yml"
                    } else {
                        false
                    }
                }
                Err(_) => false,
            })
        {
            Manifest::new_from_reader(file_path)
        } else {
            Err(anyhow!("Could not find manifest file in archive"))
        }
    }

    pub fn pack(self) -> Result<String> {
        let mut logger = Logger::new();
        let package_path = self.get_package_path_buf()?;
        let manifest_path = self.get_manifest_path_buf(&package_path)?;

        logger.loading("Loading package manifest");
        let manifest = Manifest::new_from_path(manifest_path.to_path_buf())?;
        logger.info("Loaded package manifest.").newline(1);

        let archive_file_location = format!("{}.hpkg", manifest.id);
        let archive_file = fs::File::create(&archive_file_location)?;
        let encoder = GzEncoder::new(archive_file, Compression::best());
        let mut builder = Builder::new(encoder);

        logger.info(format!("Packaging {}", package_path.display()));
        let mut mappings = Vec::new();
        for mut file_mapping_definition in manifest.mappings {
            file_mapping_definition.set_integrity(package_path.to_path_buf())?;
            let file_path = package_path.join(&file_mapping_definition.i);
            builder.append_path_with_name(&file_path, &file_mapping_definition.i)?;
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
        builder.append_path_with_name(&manifest_path, "hermione.yml")?;
        builder.finish()?;
        builder.into_inner()?;
        logger.indent(1).log(format!(
            "Added <blue>{}</> to package archive",
            manifest_path
                .file_name()
                .unwrap_or(std::ffi::OsStr::new("hermione.yml"))
                .to_string_lossy()
        ));

        let loc = fs::canonicalize(Path::new(&archive_file_location))?;
        Ok(format!("{}", loc.to_string_lossy()))
    }

    pub fn unpack(self, dest: PathBuf) -> Result<PathBuf> {
        let mut logger = Logger::new();
        logger.loading(format!(
            "Starting unpacking package path {}",
            &self.package_path_buf.display()
        ));

        let archive_manifest_file = self.get_manifest_from_archive();

        let archive_file = fs::File::open(&self.package_path_buf)?;
        let decoder = GzDecoder::new(archive_file);
        let mut archive = Archive::new(decoder);

        match archive_manifest_file {
            Ok(manifest_file) => {
                let final_dest = dest.join(manifest_file.id);
                archive.unpack(&final_dest)?;
                logger.success("Done");
                Ok(final_dest)
            }
            Err(e) => Err(anyhow!("Could not unpack archive | {}", e)),
        }
    }
}
