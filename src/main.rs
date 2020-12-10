#![forbid(unsafe_code)]

use clap::{App, Arg, SubCommand};
use color_eyre::eyre::{eyre, Result};
use paris::Logger;

mod action;
mod actions;
mod config;
mod downloaded_package;
mod downloader;
mod file_mapping;
mod file_mapping_definition;
mod hooks;
mod installed_package;
mod manifest;
mod package_service;
mod packer;
mod repositories;
mod scaffold;

use crate::action::Action;
use crate::package_service::PackageService;

fn main() -> Result<()> {
    color_eyre::install()?;
    let matches = App::new("herm")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
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
                .about("completely remove Hermione packages from the system")
                .version(env!("CARGO_PKG_VERSION"))
                .author(env!("CARGO_PKG_AUTHORS"))
                .arg(
                    Arg::with_name("confirm")
                        .help("confirms your choice to implode, warning all hermione packages will be removed")
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
            SubCommand::with_name("package")
                .about("creates a package archive")
                .version(env!("CARGO_PKG_VERSION"))
                .author(env!("CARGO_PKG_AUTHORS"))
                .alias("pack")
                .arg(
                    Arg::with_name("PACKAGE_PATH")
                        .help("path to package")
                        .required(true)
                        .takes_value(true)
                        .value_name("PACKAGE_PATH")
                        .default_value(".")
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
        .subcommand(
            SubCommand::with_name("update")
                .about("update all remote repos")
                .version(env!("CARGO_PKG_VERSION"))
                .author(env!("CARGO_PKG_AUTHORS")),
        )
        .subcommand(
            SubCommand::with_name("repo")
                .about("manage repositories")
                .version(env!("CARGO_PKG_VERSION"))
                .author(env!("CARGO_PKG_AUTHORS"))
                .subcommand(
                    SubCommand::with_name("add")
                    .about("add a repository")                
                    .author(env!("CARGO_PKG_AUTHORS"))
                    .version(env!("CARGO_PKG_VERSION"))
                    .arg(
                        Arg::with_name("REPO_URL")
                        .help("add a repo")
                        .required(true)
                        .value_name("REPO_URL")
                        .takes_value(true)
                        .index(1)
                    )
                )
                .subcommand(
                    SubCommand::with_name("remove")
                    .about("remove a repository")                
                    .author(env!("CARGO_PKG_AUTHORS"))
                    .version(env!("CARGO_PKG_VERSION"))
                    .arg(
                        Arg::with_name("REPO_URL")
                        .help("remove a repo")
                        .required(true)
                        .value_name("REPO_URL")
                        .takes_value(true)
                        .index(1)
                    )
                )
        )
        .get_matches();

    let subcommand_name = String::from(matches.subcommand_name().unwrap_or("error"));
    let package_service = PackageService::new()?;

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
        ("package", Some(package_matches)) => {
            let package_path = package_matches
                .value_of("PACKAGE_PATH")
                .expect("no package path provided");

            actions::package_action::PackageAction {
                package_path: String::from(package_path),
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
        ("repo", Some(repo_matches)) => match repo_matches.subcommand() {
            ("add", Some(add_repo)) => {
                let repo_url = add_repo
                    .value_of("REPO_URL")
                    .expect("Unable to read url value");

                actions::repo_action::RepoAddAction {
                    url: String::from(repo_url),
                }
                .execute(package_service)?;
            }
            ("remove", Some(remove_repo)) => {
                let repo_url = remove_repo
                    .value_of("REPO_URL")
                    .expect("Unable to read url value");

                actions::repo_action::RepoRemoveAction {
                    url: String::from(repo_url),
                }
                .execute(package_service)?;
            }
            (_, _) => {
                actions::repo_action::RepoAction {}.execute(package_service)?;
            }
        },
        ("update", Some(_update_matches)) => {
            actions::update_action::UpdateAction {}.execute(package_service)?;
        }
        (subcommand, _) => {
            let mut logger = Logger::new();
            logger.error(format!("Unknown subcommand '{}'", subcommand));
            return Err(eyre!("Unknown subcommand. Try 'help'"));
        }
    };

    if subcommand_name == "implode" {
        Ok(())
    } else {
        match lockfile.release() {
            Ok(_) => Ok(()),
            Err(e) => Err(eyre!("Unable to release lockfile because: {}", e)),
        }
    }
}
