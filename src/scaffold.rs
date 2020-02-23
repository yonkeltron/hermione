use anyhow::{anyhow, Result};
use git2::Config;
use serde::{Deserialize, Serialize};
use slog::{error, info, Logger};
use tera::{Context, Tera};

use std::fs;
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize)]
pub struct Scaffold {
    pub author: String,
    pub package_name: String,
    pub version: String,

    #[serde(skip)]
    pub template_string_value: String,
    #[serde(skip)]
    pub package_path_buf: PathBuf,
}

impl Scaffold {
    pub fn new(package_name: &str) -> Self {
        let default_author = String::from("<Author Name>");
        let author = match Config::open_default() {
            Ok(conf) => {
                let author_name = conf.get_string("user.name").unwrap_or(default_author);
                let author_email = conf.get_string("user.email").unwrap_or(String::from(""));
                format!("{} <{}>", author_name, author_email)
            }
            Err(_) => default_author,
        };

        let template_string_value =
            String::from(include_str!("../templates/hermione_manifest_tpl.yml"));

        let package_path = Path::new(&package_name);
        let parsed_package_name = match package_path.file_stem() {
            Some(stem) => String::from(stem.to_string_lossy()),
            None => String::from("<Package Name>"),
        };

        Scaffold {
            author: author,
            package_name: String::from(parsed_package_name),
            version: String::from(env!("CARGO_PKG_VERSION")),
            template_string_value,
            package_path_buf: package_path.to_path_buf(),
        }
    }

    pub fn create_package(&self, logger: &Logger) -> Result<()> {
        info!(logger, "Creating package directory";
            "package" => &self.package_name,
            "operation" => "scaffold",
        );

        match fs::create_dir_all(&self.package_path_buf) {
            Ok(_) => {
                info!(
                    logger,
                    "Successfully created package directory";
                    "package" => self.package_name.clone(),
                    "operation" => "scaffold",
                );
                self.create_manifest(self.package_path_buf.to_path_buf(), logger)?;
                Ok(())
            }
            Err(e) => {
                error!(logger,
                    "Could not create package directory";
                    "path" => self.package_path_buf.to_str().clone(),
                    "package" => self.package_name.clone(),
                    "operation" => "scaffold",
                );
                Err(anyhow!(e))
            }
        }
    }

    pub fn create_manifest(&self, path: PathBuf, logger: &Logger) -> Result<()> {
        info!(
            logger,
            "Creating manifest file";
            "package" => self.package_name.clone(),
            "operation" => "scaffold",
        );

        let mut tera = Tera::default();
        tera.add_raw_template("hermione_manifest_tpl.yml", &self.template_string_value)?;
        let mut context = Context::new();
        context.insert("scaffold", &self);

        let hermione_string = tera.render("hermione_manifest_tpl.yml", &context)?;
        let hermione_manifest_path = path.join("hermione.yml");

        if hermione_manifest_path.is_file() {
            error!(
                logger,
                "hermione.yml already exists in current directory, will not overwrite";
                "package" => self.package_name.clone(),
                "operation" => "scaffold",
                "hermione_manifest_path" => hermione_manifest_path.to_str(),
            );
            Err(anyhow!("hermione.yml exists in current directory"))
        } else {
            fs::write(&hermione_manifest_path, hermione_string)?;
            info!(
                logger,
                "Successfully created hermione manifest file";
                "package" => self.package_name.clone(),
                "operation" => "scaffold",
                "hermione_manifest_path" => hermione_manifest_path.to_str(),
            );
            Ok(())
        }
    }
}
