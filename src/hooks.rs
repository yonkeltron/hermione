use anyhow::{anyhow, Result};
use duckscript::{runner, types};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Hooks {
    pub pre_install: Option<String>,
    pub post_install: Option<String>,
    pub pre_remove: Option<String>,
    pub post_remove: Option<String>,
    pub pre_upgrade: Option<String>,
    pub post_upgrade: Option<String>,
}

impl Hooks {
    pub fn execute_pre_install(&self) -> Result<()> {
        Hooks::execute("pre_install", &self.pre_install)
    }
    pub fn execute_post_install(&self) -> Result<()> {
        Hooks::execute("post_install", &self.post_install)
    }

    pub fn execute_pre_upgrade(&self) -> Result<()> {
        Hooks::execute("pre_upgrade", &self.pre_install)
    }
    pub fn execute_post_upgrade(&self) -> Result<()> {
        Hooks::execute("post_upgrade", &self.post_install)
    }

    pub fn execute_pre_remove(&self) -> Result<()> {
        Hooks::execute("pre_remove", &self.pre_install)
    }
    pub fn execute_post_remove(&self) -> Result<()> {
        Hooks::execute("post_remove", &self.post_install)
    }

    pub fn execute(hook_name: &str, script_string: &Option<String>) -> Result<()> {
        let mut context = types::runtime::Context::new();
        match duckscriptsdk::load(&mut context.commands) {
            Ok(_) => match script_string {
                Some(f) => match runner::run_script(f.as_str(), context) {
                    Ok(_) => Ok(()),
                    Err(e) => Err(anyhow!(
                        "Failed to run Duckscript in {} hook: {}",
                        hook_name,
                        e
                    )),
                },
                None => Ok(()),
            },
            Err(e) => Err(anyhow!(
                "Failed to run Duckscript in {} hook: {}",
                hook_name,
                e
            )),
        }
    }
}
