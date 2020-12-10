use clap::{App, Arg, ArgMatches, SubCommand};

pub fn get_matches() -> ArgMatches<'static> {
  App::new("herm")
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
      SubCommand::with_name("publish")
        .about("generate a repo from a directory of packages")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .arg(
          Arg::with_name("PACKAGES_DIR")
            .takes_value(true)
            .value_name("PACKAGES_DIR")
            .help("path to directory containing package files")
            .required(true)
            .index(1),
        )
        .arg(
          Arg::with_name("URL_PREFIX")
            .short("u")
            .long("url-prefix")
            .takes_value(true)
            .value_name("URL")
            .help("hosting URL where the package files will be located")
            .required(true),
        )
        .arg(
          Arg::with_name("REPO_FILE")
            .short("f")
            .long("file")
            .takes_value(true)
            .value_name("REPO_FILE")
            .help("name of repository file to generate")
            .default_value("repo.hermione.toml"),
        ),
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
                .index(1),
            ),
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
                .index(1),
            ),
        ),
    )
    .get_matches()
}
