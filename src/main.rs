use anyhow::{anyhow, Result};
use clap::{App, Arg, SubCommand};

use std::fs;

mod config;
mod file_mapping;
mod manifest;
mod package;

use crate::config::Config;
use crate::package::Package;

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
            SubCommand::with_name("install")
                .about("install a Hermione package")
                .version(env!("CARGO_PKG_VERSION"))
                .author(env!("CARGO_PKG_AUTHORS"))
                .arg(
                    Arg::with_name("SOURCE")
                        .help("pointer to package (git URL or local file path)")
                        .required(true)
                        .index(1),
                ),
        )
        .subcommand(
            SubCommand::with_name("remove")
                .about("removes Hermione entirely")
                .version(env!("CARGO_PKG_VERSION"))
                .author(env!("CARGO_PKG_AUTHORS")),
        )
        .get_matches();

    let config = Config::load()?;

    match matches.subcommand() {
        ("init", _init_matches) => {
            println!("{:#?}", config);
            fs::create_dir_all(config.hermione_home)?;
        }
        ("remove", _remove_matches) => {
            fs::remove_dir_all(config.hermione_home)?;
        }
        ("install", install_matches) => {
            let package_source = install_matches
                .expect("No arg matches for install")
                .value_of("SOURCE")
                .expect("Unable to read source");
            let package = Package::new_from_source(String::from(package_source), config)?;
            package.install()?;
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
