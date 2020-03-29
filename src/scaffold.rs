use anyhow::{anyhow, Result};
use git2::Config;
use serde_yaml;
use slog::{error, info, Logger};

use std::fs;
use std::path::{Path, PathBuf};

use crate::file_mapping_definition::FileMappingDefinition;
use crate::manifest::Manifest;

/// Scaffold is responsible for creating `hermione.yml` files and new Hermione package directories
pub struct Scaffold {
    pub manifest: Manifest,
    pub package_path_buf: PathBuf,
}

impl Scaffold {
    /// Returns an instance of Scaffold with a default Manifest for creating a `hermione.yml` file.
    pub fn new(package_name: &str, package_id: &str) -> Self {
        let default_author = String::from("<Author Name>");
        let default_description = format!(
            "Manifest file generated by Hermione ({}). Visit the docs at https://hermione.dev",
            env!("CARGO_PKG_VERSION")
        );
        let author = match Config::open_default() {
            Ok(conf) => {
                let author_name = conf.get_string("user.name").unwrap_or(default_author);
                let author_email = conf
                    .get_string("user.email")
                    .unwrap_or_else(|_| String::from(""));
                format!("{} <{}>", author_name, author_email)
            }
            Err(_) => default_author,
        };

        let package_path = Path::new(&package_name);
        let parsed_package_name = match package_path.file_stem() {
            Some(stem) => String::from(stem.to_string_lossy()),
            None => String::from("<Package Name>"),
        };

        Self {
            package_path_buf: package_path.to_path_buf(),
            manifest: Manifest {
                author,
                name: parsed_package_name,
                description: default_description,
                id: String::from(package_id),
                mappings: vec![
                    FileMappingDefinition::new(
                        String::from("sample.txt"),
                        String::from("{{HOME}}/sample.txt"),
                        None,
                    ),
                    FileMappingDefinition::new(
                        String::from("config.toml"),
                        String::from("/tmp/absolute/path/to/dir/config.toml"),
                        Some(String::from("unix")),
                    ),
                ],
            },
        }
    }

    /// Creates a package directory with a sample `hermione.yml` file and a couple of sample files to correspond with it.
    pub fn create_package(&self, logger: &Logger) -> Result<()> {
        info!(logger, "Creating package directory";
            "package" => &self.manifest.name,
            "operation" => "scaffold",
        );

        match fs::create_dir_all(&self.package_path_buf) {
            Ok(_) => {
                info!(
                    logger,
                    "Successfully created package directory";
                    "package" => self.manifest.name.clone(),
                    "operation" => "scaffold",
                );
                self.create_manifest(self.package_path_buf.to_path_buf(), logger)?;
                self.create_example_files(logger)?;
                Ok(())
            }
            Err(e) => {
                error!(logger,
                    "Could not create package directory";
                    "path" => self.package_path_buf.to_str(),
                    "package" => self.manifest.name.clone(),
                    "operation" => "scaffold",
                );
                Err(anyhow!(e))
            }
        }
    }

    /// Creates the `hermione.yml` file from the Manifest struct and writes it to the given PathBuf.
    ///
    /// ### Arguments
    ///
    /// * path - PathBuf of where to write the `hermione.yml` file
    /// * logger - Borrowed Logger instance
    ///
    /// Returns an Empty Result.
    pub fn create_manifest(&self, path: PathBuf, logger: &Logger) -> Result<()> {
        info!(
            logger,
            "Creating manifest file";
            "package" => self.manifest.name.clone(),
            "operation" => "scaffold",
        );

        let hermione_string = serde_yaml::to_string(&self.manifest)?;
        let hermione_manifest_path = path.join("hermione.yml");

        if hermione_manifest_path.is_file() {
            error!(
                logger,
                "hermione.yml already exists in current directory, will not overwrite";
                "package" => self.manifest.name.clone(),
                "operation" => "scaffold",
                "hermione_manifest_path" => hermione_manifest_path.to_str(),
            );
            Err(anyhow!("hermione.yml exists in current directory"))
        } else {
            fs::write(&hermione_manifest_path, hermione_string)?;
            info!(
                logger,
                "Successfully created hermione manifest file";
                "package" => self.manifest.name.clone(),
                "operation" => "scaffold",
                "hermione_manifest_path" => hermione_manifest_path.to_str(),
            );
            Ok(())
        }
    }

    /// Creates the Hermione sample files.
    ///
    /// ### Arguments
    ///
    /// * logger - Borrowed Logger instance
    ///
    /// Returns an Empty Result.
    fn create_example_files(&self, logger: &Logger) -> Result<()> {
        info!(logger, "Creating example files";
        "package" => self.manifest.name.clone(),
        "operation" => "scaffold");

        let sample_file_path = self.package_path_buf.join("sample.txt");
        let other_file_path = self.package_path_buf.join("config.toml");
        fs::write(
            sample_file_path,
            String::from("Sample Text File Generated by Hermione"),
        )?;
        fs::write(
            other_file_path,
            String::from("description = \"Sample Config File Generated by Hermione\""),
        )?;

        info!(logger, "Finished creating example files";
        "package" => self.manifest.name.clone(),
        "operation" => "scaffold");
        Ok(())
    }
}
