use anyhow::{anyhow, Result};
use clap::{App, Arg, SubCommand};
use slog::{error, o, Level};

mod action;
mod actions;
mod downloaded_package;
mod file_mapping;
mod file_mapping_definition;
mod git_downloader;
mod hooks;
mod installed_package;
mod logger;
mod manifest;
mod package_service;
mod scaffold;

use crate::action::Action;
use crate::logger::create_logger;
use crate::package_service::PackageService;

fn main() -> Result<()> {
    let matches = App::new("herm")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(
            Arg::with_name("log_format")
                .long("log-format")
                .help("output log format")
                .possible_values(&["human", "json", "prettyjson"])
                .default_value("human")
                .global(true),
        )
        .subcommand(
            SubCommand::with_name("init")
                .about("initialize Hermione manifest file")
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
        .subcommand(
            SubCommand::with_name("new")
                .about("generate new Hermione package")
                .version(env!("CARGO_PKG_VERSION"))
                .author(env!("CARGO_PKG_AUTHORS"))
                .arg(
                    Arg::with_name("PACKAGE_NAME")
                        .help("package name")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::with_name("PACKAGE_ID")
                        .help("package reverse domain id <com.example.package>")
                        .short("i")
                        .long("id")
                        .takes_value(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("upgrade")
                .about("upgrade an existing")
                .version(env!("CARGO_PKG_VERSION"))
                .author(env!("CARGO_PKG_AUTHORS"))
                .arg(
                    Arg::with_name("PACKAGE_NAMES")
                        .help("package names")
                        .multiple(true)
                        .default_value("")
                        .index(1),
                ),
        )
        .get_matches();

    let format = matches
        .value_of("log_format")
        .expect("Unable to read log format");
    let subcommand_name = String::from(matches.subcommand_name().unwrap_or("error"));
    let log = create_logger(format, Level::Info).new(o!("action" => subcommand_name.clone()));
    let package_service = PackageService::new(log)?;

    let lockfile = package_service.lockfile()?;

    if subcommand_name != "implode" {
        package_service.init()?;
    };

    match matches.subcommand() {
        ("init", Some(_init_matches)) => {
            actions::init_action::InitAction {}.execute(package_service)?;
        }
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
        ("new", Some(new_matches)) => {
            let package_name = new_matches
                .value_of("PACKAGE_NAME")
                .expect("No package name provided");
            let package_id = new_matches
                .value_of("PACKAGE_ID")
                .unwrap_or("com.example.package");

            actions::new_action::NewAction {
                package_name: String::from(package_name),
                package_id: String::from(package_id),
            }
            .execute(package_service)?;
        }
        ("upgrade", Some(upgrade_matches)) => {
            let package_names = upgrade_matches
                .values_of("PACKAGE_NAMES")
                .expect("Unable to read package names");

            actions::upgrade_action::UpgradeAction {
                package_names: package_names
                    .map(String::from)
                    .filter(|s| !s.is_empty())
                    .collect(),
            }
            .execute(package_service)?;
        }
        (subcommand, _) => {
            error!(
                package_service.logger,
                "Unknown subcommand '{}'", subcommand
            );
            return Err(anyhow!("Unknown subcommand. Try 'help'"));
        }
    };

    if subcommand_name == "implode" {
        Ok(())
    } else {
        match lockfile.release() {
            Ok(_) => Ok(()),
            Err(e) => Err(anyhow!("Unable to release lockfile because: {}", e)),
        }
    }
}
