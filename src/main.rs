use anyhow::Result;
use clap::{App, Arg, SubCommand};

mod action;
mod actions;
mod downloaded_package;
mod file_mapping;
mod file_mapping_definition;
mod installed_package;
mod manifest;
mod package_service;

use crate::action::Action;
use crate::installed_package::InstalledPackage;
use crate::package_service::PackageService;

fn main() -> Result<()> {
    let matches = App::new("herm")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
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

    let package_service = PackageService::new()?;
    package_service.init()?;

    match matches.subcommand() {
        ("install", Some(install_matches)) => {
            let package_source = install_matches
                .value_of("SOURCE")
                .expect("Unable to read source");

            actions::install_action::InstallAction {
                package_source: String::from(package_source),
            }
            .execute(package_service)?;
        }
        ("implode", Some(implode_matches)) => {
            let confirmed = implode_matches.is_present("confirm");
            actions::implode_action::ImplodeAction {
                yes_i_am_sure: confirmed,
            }
            .execute(package_service)?;
        }
        ("list", _list_matches) => {
            actions::list_action::ListAction {}.execute(package_service)?;
        }
        ("remove", Some(remove_matches)) => {
            let package_name = remove_matches
                .value_of("PACKAGE")
                .expect("unable to read package name");

            let name = String::from(package_name);
            actions::remove_action::RemoveAction { package_name: name }.execute(package_service)?;
        }
        (subcommand, _) => eprintln!("Unknown subcommand '{}'", subcommand),
    };

    Ok(())
}
