use anyhow::Result;
use clap::{App, Arg, SubCommand};

mod downloaded_package;
mod file_mapping;
mod file_mapping_definition;
mod installed_package;
mod manifest;
mod package_service;

use crate::installed_package::InstalledPackage;
use crate::package_service::PackageService;

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
            SubCommand::with_name("implode")
                .about("completely remove Hermione from the system")
                .version(env!("CARGO_PKG_VERSION"))
                .author(env!("CARGO_PKG_AUTHORS"))
                .arg(
                    Arg::with_name("confirm")
                        .help("pointer to package (git URL or local file path)")
                        .long("yes-i-am-sure")
                        .required(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("list")
                .about("lists installed Hermione packages")
                .alias("ls")
                .version(env!("CARGO_PKG_VERSION"))
                .author(env!("CARGO_PKG_AUTHORS")),
        )
        .subcommand(
            SubCommand::with_name("remove")
                .about("removes Hermione entirely")
                .version(env!("CARGO_PKG_VERSION"))
                .author(env!("CARGO_PKG_AUTHORS"))
                .alias("uninstall")
                .arg(
                    Arg::with_name("PACKAGE")
                        .help("name of installed package")
                        .required(true)
                        .index(1),
                ),
        )
        .get_matches();

    match matches.subcommand() {
        ("init", _init_matches) => {
            PackageService::new()?.init()?;
        }
        ("install", Some(install_matches)) => {
            let package_source = install_matches
                .value_of("SOURCE")
                .expect("Unable to read source");
            PackageService::download_and_install(String::from(package_source))?;
        }
        ("implode", Some(implode_matches)) => {
            let confirmed = implode_matches.is_present("confirm");
            if confirmed {
                let ps = PackageService::new()?;
                ps.implode()?;
            } else {
                println!("I am not sure you want me to do this.");
                println!("Please pass confirm flag if you are sure");
            }
        }
        ("list", _list_matches) => {
            let installed_packages = PackageService::new()?.list_installed_packages()?;

            println!("Displaying: {} Packages", installed_packages.len());
            installed_packages
                .iter()
                .for_each(|installed_package| println!("{}", installed_package.package_name));
        }
        ("remove", Some(remove_matches)) => {
            let package_name = remove_matches
                .value_of("PACKAGE")
                .expect("unable to read package name");

            let name = String::from(package_name);
            let remove_result = match InstalledPackage::from_package_name(name.clone()) {
                Ok(package) => package.remove(),
                Err(e) => Err(e),
            };

            match remove_result {
                Ok(_success) => println!("Removed package {}", name),
                Err(e) => eprintln!("Unable to remove {} because {}", name, e.to_string()),
            }
        }
        (subcommand, _) => eprintln!("Unknown subcommand '{}'", subcommand),
    };

    Ok(())
}
