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

    pub fn repo_list(&self) -> Vec<String> {
        self.repository_urls.to_vec()
    }

    pub fn add_repo_url(self, repo_url: String) -> Self {
        let found_duplicate = self
            .repository_urls
            .to_vec()
            .into_iter()
            .find(|url| url.eq(&repo_url));
        let repository_urls = match found_duplicate {
            Some(_) => self.repository_urls,
            None => {
                let mut new = self.repository_urls.to_vec();
                new.push(repo_url);
                new
            }
        };
        Self {
            repository_urls,
            ..self
        }
    }

    pub fn remove_repo_url(self, repo_url: String) -> Self {
        let mut repository_urls = self.repository_urls.to_vec();
        repository_urls.retain(|url| !url.eq(&repo_url));
        Self {
            repository_urls,
            ..self
        }
    }

    pub fn fetch_and_build_index(&self) -> Result<PackageIndex> {
        let client = Client::builder().timeout(Duration::from_secs(7)).build()?;
        let mut logger = Logger::new();

        logger.loading("Fetching repositories...");

        let available_repositories = self
            .repository_urls
            .iter()
            .map(|repository_url| RemoteRepository::new(repository_url))
            .filter(|res| res.is_ok())
            .map(|ok_res| ok_res.expect("Unable to instantiate remote repository"))
            .map(|remote_respository| remote_respository.download_contents(&client));

        logger.info("Finished repository fetch attempt.");

        logger.loading("Building package index...");
        let combined_index = available_repositories
            .filter(|res| res.is_ok())
            .map(|ok_res| ok_res.expect("Unable to build index from successful download"))
            .map(|repository_contents| repository_contents.to_index())
            .fold(HashMap::new(), |a, b| a.into_iter().chain(b).collect());
        logger.info("Built package index.");

        Ok(combined_index)
    }
}
