use color_eyre::eyre::{eyre, Result, WrapErr};
use paris::Logger;
use serde::{Deserialize, Serialize};

use std::time::Duration;

use crate::repositories::repository_contents::RepositoryContents;

#[derive(Serialize, Deserialize)]
pub struct HermioneConfig {
    repository_urls: Vec<String>,
}

/// `MyConfig` implements `Default`
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

    pub fn available_repositories(&self) -> Result<Vec<RepositoryContents>> {
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(7))
            .build()?;

        let available_repositories = self
            .repository_urls
            .iter()
            .map(|repository_url| {
                let mut logger = Logger::new();
                logger.loading(format!("Fetching repository {}", repository_url));

                let result = match client.get(repository_url).send() {
                    Ok(response) => {
                        if response.status().is_success() {
                            match response.text() {
                                Ok(text) => Ok(toml::from_str::<RepositoryContents>(&text)
                                    .wrap_err_with(|| format!("Unable to deserialize TOML"))),
                                Err(e) => {
                                    Err(eyre!("Unable to decode response text to UTF-8: {}", e))
                                }
                            }
                        } else {
                            Err(eyre!(
                                "HTTP request ({}) failed with status code {}",
                                repository_url,
                                response.status().as_str()
                            ))
                        }
                    }
                    Err(err) => Err(eyre!(
                        "Unable to fetch repository file from server: {}",
                        err
                    )),
                };

                if result.is_ok() {
                    logger.success(format!("Fetched repository from {}", repository_url));
                } else {
                    logger.warn(format!(
                        "Failed to fetch repository from {}",
                        repository_url
                    ));
                };

                result
            })
            .filter(|res| res.is_ok())
            .map(|ok_res| ok_res.unwrap());

        let mut logger = Logger::new();

        logger.info("Finished repository fetch attempt.");

        let repos = available_repositories
            .filter(|res| res.is_ok())
            .map(|ok_res| ok_res.unwrap())
            .collect::<Vec<RepositoryContents>>();

        Ok(repos)
    }
}
