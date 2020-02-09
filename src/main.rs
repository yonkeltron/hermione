use anyhow::Result;
use clap::{App, Arg, SubCommand};

mod config;
mod downloaded_package;
mod file_mapping;
mod file_mapping_definition;
mod installed_package;
mod manifest;
mod package;

use crate::config::Config;
use crate::installed_package::InstalledPackage;
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
                .author(env!("CARGO_PKG_AUTHORS"))
                .arg(
                    Arg::with_name("PACKAGE")
                        .help("name of installed package")
                        .required(true)
                        .index(1),
                ),
        )
        .get_matches();

    let config = Config::load()?;

    match matches.subcommand() {
        ("init", _init_matches) => {
            config.init_hermione_home()?;
        }
        ("remove", Some(remove_matches)) => {
            let package_name = remove_matches
                .value_of("PACKAGE")
                .expect("unable to read package name");

            let name = String::from(package_name);
            let remove_result =
                match InstalledPackage::from_package_name(name.clone()) {
                    Ok(package) => package.remove(),
                    Err(e) => Err(e),
                };

            match remove_result {
                Ok(_success) => println!("Removed package {}", name),
                Err(e) => eprintln!("Unable to remove {} because {}", name, e.to_string()),
            }
        }
        ("install", Some(install_matches)) => {
            let package_source = install_matches
                .value_of("SOURCE")
                .expect("Unable to read source");
            let package = Package::download(String::from(package_source))?;
            package.install()?;
        }
        (subcommand, _) => eprintln!("Unknown subcommand '{}'", subcommand),
    };

    Ok(())
}
