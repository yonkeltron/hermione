use anyhow::{anyhow, Result};
use clap::{App, Arg, SubCommand};

use std::fs;

mod config;
mod file_mapping;
mod manifest;
mod package;

use crate::config::Config;

fn main() -> Result<()> {
    let matches = App::new("herm")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .subcommand(
            SubCommand::with_name("init")
                .about("initializes Hermione")
                .version(env!("CARGO_PKG_VERSION"))
                .author(env!("CARGO_PKG_AUTHORS")),
        )
        .subcommand(
            SubCommand::with_name("remove")
                .about("removes Hermione entirely")
                .version(env!("CARGO_PKG_VERSION"))
                .author(env!("CARGO_PKG_AUTHORS")),
        )
        .get_matches();

    match matches.subcommand() {
        ("init", _init_matches) => {
            let config = Config::load()?;
            println!("{:#?}", config);
            fs::create_dir_all(config.hermione_home)?;
        }
        ("remove", _remove_matches) => {
            let config = Config::load()?;
            fs::remove_dir_all(config.hermione_home)?;
        }
        (subcommand, _) => eprintln!("Unknown subcommand '{}'", subcommand),
    };

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
