use color_eyre::eyre::{eyre, Result, WrapErr};
use paris::Logger;
use reqwest::blocking::Client;
use url::Url;

use crate::repositories::repository_contents::RepositoryContents;

pub struct RemoteRepository {
    url: Url,
}

impl RemoteRepository {
    pub fn new(raw_url: &str) -> Result<Self> {
        let url = Url::parse(raw_url).wrap_err("Invalid repo URL supplied")?;

        Ok(Self { url })
    }

    pub fn download_contents(self, client: &Client) -> Result<RepositoryContents> {
        let repository_url = self.url;
        let mut logger = Logger::new();
        logger.loading(format!("Fetching repository {}", repository_url));

        let result = match client.get(repository_url.as_str()).send() {
            Ok(response) => {
                if response.status().is_success() {
                    match response.text() {
                        Ok(text) => Ok(toml::from_str::<RepositoryContents>(&text)
                            .wrap_err_with(|| format!("Unable to deserialize TOML"))),
                        Err(e) => Err(eyre!("Unable to decode response text to UTF-8: {}", e)),
                    }
                } else {
                    Err(eyre!(
                        "Web request ({}) failed with status code {}",
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

        result?
    }
}
