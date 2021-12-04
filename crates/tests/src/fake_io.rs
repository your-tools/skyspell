use std::cell::RefCell;
use std::collections::HashMap;

use anyhow::{anyhow, Result};

use skyspell_core::OperatingSystemIO;

pub struct FakeIO {
    env: HashMap<String, String>,
    stdout: RefCell<String>,
}

impl Default for FakeIO {
    fn default() -> Self {
        Self::new()
    }
}

impl FakeIO {
    pub fn new() -> Self {
        Self {
            env: HashMap::new(),
            stdout: RefCell::new(String::new()),
        }
    }

    pub fn get_output(self) -> String {
        self.stdout.borrow().to_string()
    }

    pub fn set_env_var(&mut self, key: &str, value: &str) {
        self.env.insert(key.to_string(), value.to_string());
    }
}

impl OperatingSystemIO for FakeIO {
    fn get_env_var(&self, key: &str) -> Result<String> {
        let res = self
            .env
            .get(key)
            .ok_or_else(|| anyhow!("No such key: {}", key))?;
        Ok(res.to_owned())
    }

    fn print(&self, text: &str) {
        self.stdout.borrow_mut().push_str(text)
    }
}
