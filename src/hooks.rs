use anyhow::{anyhow, Result};
use duckscript::{runner, types};
use serde::{Deserialize, Serialize};
use slog::{info, o, Logger};

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
    pub fn execute_pre_install(&self, logger: &Logger) -> Result<()> {
        Hooks::execute("pre_install", &self.pre_install, logger)
    }
    pub fn execute_post_install(&self, logger: &Logger) -> Result<()> {
        Hooks::execute("post_install", &self.post_install, logger)
    }

    pub fn execute_pre_upgrade(&self, logger: &Logger) -> Result<()> {
        Hooks::execute("pre_upgrade", &self.pre_install, logger)
    }
    pub fn execute_post_upgrade(&self, logger: &Logger) -> Result<()> {
        Hooks::execute("post_upgrade", &self.post_install, logger)
    }

    pub fn execute_pre_remove(&self, logger: &Logger) -> Result<()> {
        Hooks::execute("pre_remove", &self.pre_install, logger)
    }
    pub fn execute_post_remove(&self, logger: &Logger) -> Result<()> {
        Hooks::execute("post_remove", &self.post_install, logger)
    }

    pub fn execute(hook_name: &str, script_string: &Option<String>, logger: &Logger) -> Result<()> {
        match script_string {
            Some(f) => {
                let hogger = logger.new(o!("hook" => String::from(hook_name)));
                info!(hogger, "Initiated {} hook", hook_name);
                info!(hogger, "Loading Duckscript sdk into context");

                let mut context = types::runtime::Context::new();
                match duckscriptsdk::load(&mut context.commands) {
                    Ok(_) => match runner::run_script(f.as_str(), context) {
                        Ok(_) => {
                            info!(hogger, "Finished running Duckscript");
                            Ok(())
                        }
                        Err(e) => Err(anyhow!(
                            "Failed to run Duckscript in {} hook: {}",
                            hook_name,
                            e
                        )),
                    },
                    Err(e) => Err(anyhow!(
                        "Failed to run Duckscript in {} hook: {}",
                        hook_name,
                        e
                    )),
                }
            }
            None => Ok(()),
        }
    }
}
