use anyhow::Result;
use fs_extra::dir;
use git2::Repository;

use std::path::Path;

use crate::config::Config;
use crate::downloaded_package::DownloadedPackage;

pub struct Package {}

impl Package {
    pub fn download(src: String, config: &Config) -> Result<DownloadedPackage> {
        let path = Path::new(&src).canonicalize()?;
        let package_name = Self::source_to_package_name(&src);
        let checkout_path = Self::install_path(&config.hermione_home, &package_name);
        let dest_path = Path::new(&checkout_path);

        let local_path = if path.is_dir() {
            let options = dir::CopyOptions::new();
            dir::copy(&path, &config.hermione_home, &options)?;
            checkout_path
        } else {
            let repo = Repository::clone(&src, dest_path)?;
            let package_name = repo.path().display();
            format!("{}", package_name)
        };

        Ok(DownloadedPackage {
            local_path: Path::new(&local_path).to_path_buf(),
        })
    }

    fn source_to_package_name(src: &str) -> String {
        let path = Path::new(src);

        let package_name = match path.file_stem() {
            Some(stem) => String::from(stem.to_string_lossy()),
            None => String::from("UNKNOWN_PACKAGE"),
        };

        package_name
    }

    fn install_path(hermione_home: &str, package_name: &str) -> String {
        let path = Path::new(hermione_home).join(package_name);

        String::from(path.to_string_lossy())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_to_package_name_with_url() {
        let input = "http://github.com/panda/bamboo.git";
        let expected = String::from("bamboo");

        assert_eq!(Package::source_to_package_name(input), expected);
    }

    #[test]
    fn test_source_to_package_name_with_local_path() {
        let input = "./panda";
        let expected = String::from("panda");

        assert_eq!(Package::source_to_package_name(input), expected);
    }

    #[test]
    fn test_download() {
        let src = String::from("./example-package");

        let config = Config::load().expect("Unable to load config in test");
        config
            .init_hermione_home()
            .expect("Unable to init Hermione home in test");

        let package = Package::download(src, &config).expect("Unable to instantiate package");
        assert!(package.local_path.is_dir());
        package.remove().expect("Unable to clean up after test");
    }

    #[test]
    fn test_install_path() {
        let package_name = "panda";
        let hermione_home = "bamboo";

        let actual = Package::install_path(hermione_home, package_name);

        let expected = String::from("bamboo/panda");

        assert_eq!(expected, actual);
    }
}
