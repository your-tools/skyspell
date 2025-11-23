use anyhow::{Result, anyhow};

pub trait OperatingSystemIO {
    fn get_env_var(&self, key: &str) -> Result<String>;
    fn print(&self, text: &str);
}

#[derive(Copy, Clone)]
pub struct StandardIO;

impl OperatingSystemIO for StandardIO {
    fn get_env_var(&self, key: &str) -> Result<String> {
        std::env::var(key).map_err(|_| anyhow!("{key} not found in environment"))
    }

    fn print(&self, text: &str) {
        print!("{text}");
    }
}
