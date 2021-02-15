use color_eyre::eyre::{eyre, Result};
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

    /// Given package path, verify manifest file exists, return manifest PathBuf if present
    fn get_manifest_path_buf(&self, package_path: &PathBuf) -> Result<PathBuf> {
        let manifest_path = package_path.join(Manifest::manifest_file_name());
        if manifest_path.is_file() {
            Ok(manifest_path)
        } else {
            Err(eyre!(
                "Manifest file does not exist in directory {}",
                package_path.display()
            ))
        }
    }

    pub fn get_manifest_from_archive(&self) -> Result<Manifest> {
        let archive_file = fs::File::open(&self.package_path_buf)?;
        let decoder = GzDecoder::new(archive_file);
        let mut archive = Archive::new(decoder);
        if let Some(file_path) = archive
            .entries()?
            .filter_map(|entry| entry.ok())
            .find(|entry| match entry.path() {
                Ok(path) => {
                    if let Some(file_name) = path.file_name() {
                        file_name == Manifest::manifest_file_name().as_str()
                    } else {
                        false
                    }
                }
                Err(_) => false,
            })
        {
            Manifest::new_from_reader(file_path)
        } else {
            Err(eyre!("Could not find manifest file in archive"))
        }
    }

    pub fn pack(self) -> Result<String> {
        if self.package_path_buf.is_dir() {
            let mut logger = Logger::new();

            let manifest_path = self.get_manifest_path_buf(&self.package_path_buf)?;

            logger.loading("Loading package manifest");
            let manifest = Manifest::new_from_path(&manifest_path)?;
            logger.info("Loaded package manifest.").newline(1);

            // Create archive container for files
            let archive_file_location = format!("{}_{}.hpkg", manifest.id, manifest.version);
            let archive_file = fs::File::create(&archive_file_location)?;
            let encoder = GzEncoder::new(archive_file, Compression::best());
            let mut builder = Builder::new(encoder);

            // Loop through mappings, generate integrity and create mappings vec
            logger.info(format!("Packaging {}", self.package_path_buf.display()));
            let mut mappings = Vec::new();
            for file_mapping_definition in manifest.mappings.clone() {
                let new_file_mapping_definition = file_mapping_definition
                    .with_integrity_set(self.package_path_buf.to_path_buf())?;
                let file_path = self.package_path_buf.join(&new_file_mapping_definition.i);
                builder.append_path_with_name(&file_path, &new_file_mapping_definition.i)?;
                logger.indent(1).log(format!(
                    "Added <blue>{}</> to package archive",
                    new_file_mapping_definition.i
                ));
                mappings.push(new_file_mapping_definition);
            }

            let changed_manifest = manifest.set_mappings(mappings);
            let manifest_yaml = serde_yaml::to_string(&changed_manifest)?;
            fs::write(&manifest_path, manifest_yaml)?;
            logger.info(format!(
                "Wrote integrity data to {}",
                manifest_path.display()
            ));
            builder.append_path_with_name(&manifest_path, Manifest::manifest_file_name())?;
            builder.into_inner()?;
            logger.indent(1).log(format!(
                "Added <blue>{}</> to package archive",
                manifest_path
                    .file_name()
                    .ok_or_else(|| {
                        eyre!(
                            "Unable to add {} to package archive",
                            manifest_path.display()
                        )
                    })?
                    .to_string_lossy()
            ));

            let loc = fs::canonicalize(Path::new(&archive_file_location))?;
            Ok(format!("{}", loc.to_string_lossy()))
        } else {
            Err(eyre!(
                "Package path ({}) is not a directory",
                self.package_path_buf.display()
            ))
        }
    }

    pub fn unpack(self, dest: PathBuf) -> Result<PathBuf> {
        let mut logger = Logger::new();
        logger.loading(format!(
            "Starting unpacking package path {}",
            &self.package_path_buf.display()
        ));

        logger.info("Ingesting manifest file from archive in memory");
        let archive_manifest_file = self.get_manifest_from_archive();

        let archive_file = fs::File::open(&self.package_path_buf)?;
        let decoder = GzDecoder::new(archive_file);
        let mut archive = Archive::new(decoder);

        match archive_manifest_file {
            Ok(manifest_file) => {
                let final_dest = dest.join(manifest_file.id);
                logger.indent(1).log(format!(
                    "Unpacking package in directory: {}",
                    final_dest.display()
                ));
                archive.unpack(&final_dest)?;
                logger.success("Finished unpacking");
                Ok(final_dest)
            }
            Err(e) => Err(e),
        }
    }
}
