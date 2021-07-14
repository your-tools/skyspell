use std::cell::RefCell;
use std::collections::HashMap;

use crate::os_io::OperatingSystemIO;

use anyhow::{anyhow, Result};

pub(crate) struct FakeIO {
    env: HashMap<String, String>,
    stdout: RefCell<String>,
}

impl FakeIO {
    pub(crate) fn new() -> Self {
        Self {
            env: HashMap::new(),
            stdout: RefCell::new(String::new()),
        }
    }

    pub(crate) fn get_output(self) -> String {
        self.stdout.borrow().to_string()
    }

    pub(crate) fn set_env_var(&mut self, key: &str, value: &str) {
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
