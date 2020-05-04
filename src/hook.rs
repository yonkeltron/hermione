use anyhow::{anyhow, Context, Result};
use duckscript::{runner, types};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Hooks {
    pub pre_install_file: Option<String>,
    pub post_install_file: Option<String>,
}

impl Hooks {
    pub fn execute_pre_install(&self) -> Result<()> {
        let mut context = types::runtime::Context::new();
        match &self.pre_install_file {
            Some(f) => match runner::run_script_file(f.as_str(), context) {
                Ok(_) => Ok(()),
                Err(_) => Err(anyhow!("Failed to run duckscript")),
            },
            None => Ok(()),
        }
    }
    pub fn execute_post_install(&self) -> Result<()> {
        let mut context = types::runtime::Context::new();
        match &self.post_install_file {
            Some(f) => match runner::run_script_file(f.as_str(), context) {
                Ok(_) => Ok(()),
                Err(_) => Err(anyhow!("Failed to run duckscript")),
            },
            None => Ok(()),
        }
    }
}
