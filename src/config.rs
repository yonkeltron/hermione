use std::path::Path;

use anyhow::{anyhow, Result};
use home::home_dir;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub version: u16,
    pub hermione_home: String,
}

impl ::std::default::Default for Config {
    fn default() -> Self {
        let user_home_directory = match home_dir() {
            Some(path_buf) => {
                let canonicalized_buf = path_buf
                    .canonicalize()
                    .expect("Unable to determine user's home directory");

                let path_display = canonicalized_buf.display();

                format!("{}", path_display)
            }
            None => format!("{}", Path::new(".").display()),
        };
        Self {
            version: 0,
            hermione_home: format!("{}/{}", user_home_directory, ".local/share/hermione"),
        }
    }
}

impl Config {
    pub fn load() -> Result<Config> {
        match confy::load("hermione") {
            Ok(config) => Ok(config),
            Err(e) => Err(anyhow!("Unable to load config: {}", e)),
        }
    }
}
