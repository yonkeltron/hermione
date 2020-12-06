use color_eyre::eyre::{eyre, Result};
use duckscript::{runner, types};
use paris::Logger;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Hooks {
    pub pre_install: Option<String>,
    pub post_install: Option<String>,
    pub pre_remove: Option<String>,
    pub post_remove: Option<String>,
}

impl Hooks {
    pub fn execute_pre_install(&self) -> Result<()> {
        Hooks::execute("pre_install", &self.pre_install)
    }
    pub fn execute_post_install(&self) -> Result<()> {
        Hooks::execute("post_install", &self.post_install)
    }

    pub fn execute_pre_remove(&self) -> Result<()> {
        Hooks::execute("pre_remove", &self.pre_remove)
    }
    pub fn execute_post_remove(&self) -> Result<()> {
        Hooks::execute("post_remove", &self.post_remove)
    }

    pub fn execute(hook_name: &str, script_string: &Option<String>) -> Result<()> {
        match script_string {
            Some(f) => {
                let mut logger = Logger::new();
                logger.info(format!("Initiated {} hook", hook_name));
                logger
                    .info("Loading Duckscript sdk into context")
                    .newline(1);

                let mut context = types::runtime::Context::new();
                match duckscriptsdk::load(&mut context.commands) {
                    Ok(_) => match runner::run_script(f.as_str(), context) {
                        Ok(_) => {
                            logger.newline(1).success("Finished running Duckscript");
                            Ok(())
                        }
                        Err(e) => Err(eyre!(
                            "Failed to run Duckscript in {} hook: {}",
                            hook_name,
                            e
                        )),
                    },
                    Err(e) => Err(eyre!(
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
