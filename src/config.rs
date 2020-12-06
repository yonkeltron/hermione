use color_eyre::eyre::{eyre, Result, WrapErr};
use paris::Logger;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

use std::collections::HashMap;
use std::time::Duration;

use crate::repositories::package_index::PackageIndex;
use crate::repositories::remote_repository::RemoteRepository;

#[derive(Serialize, Deserialize)]
pub struct HermioneConfig {
    repository_urls: Vec<String>,
}

impl ::std::default::Default for HermioneConfig {
    fn default() -> Self {
        Self {
            repository_urls: vec![],
        }
    }
}

impl HermioneConfig {
    pub fn load() -> Result<Self> {
        let config: Self = confy::load("hermione")?;

        Ok(config)
    }

    pub fn store(self) -> Result<()> {
        confy::store("hermione", self)?;

        Ok(())
    }

    pub fn fetch_repos_and_build_index(&self) -> Result<PackageIndex> {
        let client = Client::builder().timeout(Duration::from_secs(7)).build()?;

        let available_repositories = self
            .repository_urls
            .iter()
            .map(|repository_url| RemoteRepository::new(repository_url))
            .filter(|res| res.is_ok())
            .map(|ok_res| ok_res.expect("Unable to instantiate remote repository"))
            .map(|remote_respository| remote_respository.download_contents(&client));

        let mut logger = Logger::new();

        logger.info("Finished repository fetch attempt.");

        let combined_index = available_repositories
            .filter(|res| res.is_ok())
            .map(|ok_res| ok_res.expect("Unable to build index from successful download"))
            .map(|repository_contents| repository_contents.to_index())
            .fold(HashMap::new(), |a, b| a.into_iter().chain(b).collect());

        Ok(combined_index)
    }
}
