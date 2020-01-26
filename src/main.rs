use anyhow::{anyhow, Result};
use clap::{App, Arg};

use std::fs;

mod config;
mod file_mapping;
mod manifest;
mod package;

use crate::config::Config;

fn main() -> Result<()> {
    let config = Config::load()?;
    fs::create_dir_all(config.hermione_home)?;

    match manifest::Manifest::new_from_file(String::from("hermione.yml")) {
        Ok(manifest) => {
            for mapping in manifest.mappings {
                println!("{}", mapping.display_line());
            }
        }
        Err(e) => eprintln!("{}", e.to_string()),
    };

    Ok(())
}
