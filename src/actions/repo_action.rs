use color_eyre::eyre::{Result, WrapErr};
use paris::Logger;
use url::Url;

use crate::action::Action;
use crate::config::HermioneConfig;
use crate::package_service::PackageService;

pub struct RepoAction {}

pub struct RepoAddAction {
    pub url: String,
}

pub struct RepoRemoveAction {
    pub url: String,
}

impl Action for RepoAction {
    fn execute(self, _package_service: PackageService) -> Result<()> {
        let mut logger = Logger::new();
        logger.info("Initialized");
        let repo_list = HermioneConfig::load()?.repo_list();
        repo_list.iter().enumerate().for_each(|(index, repo_url)| {
            logger
                .indent(1)
                .info(format!("{}. {}", (index + 1).to_string(), repo_url,));
        });
        logger.success(format!("Displayed: {} Repo", repo_list.len()));
        Ok(())
    }
}

impl Action for RepoAddAction {
    fn execute(self, _package_service: PackageService) -> Result<()> {
        let mut logger = Logger::new();
        logger.info("Initialized");

        let parsed_url = Url::parse(&self.url)
            .wrap_err_with(|| format!("Could not add Repo URL ({}), not a valid URL.", self.url))?;

        HermioneConfig::load()
            .wrap_err("Couldn't load Hermione Config")?
            .add_repo_url(parsed_url.into_string())
            .store()
            .wrap_err("Couldn't save Hermione Config")?;
        logger.success(format!("Repo: ({}) Added", self.url));
        Ok(())
    }
}

impl Action for RepoRemoveAction {
    fn execute(self, _package_service: PackageService) -> Result<()> {
        let mut logger = Logger::new();
        logger.info("Initialized");

        let parsed_url = Url::parse(&self.url).wrap_err_with(|| {
            format!("Could not remove Repo URL ({}), not a valid URL.", self.url)
        })?;

        HermioneConfig::load()
            .wrap_err("Couldn't load Hermione Config")?
            .remove_repo_url(parsed_url.into_string())
            .store()
            .wrap_err("Couldn't save Hermione Config")?;

        logger.success(format!("Repo: ({}) Removed", self.url));
        Ok(())
    }
}
