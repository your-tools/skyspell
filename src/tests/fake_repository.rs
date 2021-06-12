use anyhow::{anyhow, Result};

use std::collections::{HashMap, HashSet};
use std::path::Path;

use crate::Repository;

#[derive(Default)]
pub(crate) struct FakeRepository {
    good: HashSet<String>,
    ignored: HashSet<String>,
    skipped_file_names: HashSet<String>,
    skipped_paths: HashSet<String>,
    ignored_for_file: HashMap<String, Vec<String>>,
    ignored_for_ext: HashMap<String, Vec<String>>,
}

impl FakeRepository {
    pub(crate) fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
}

impl Repository for FakeRepository {
    fn insert_ignored_words(&mut self, words: &[&str]) -> Result<()> {
        for word in words {
            self.ignored.insert(word.to_string());
        }
        Ok(())
    }

    fn add_ignored(&mut self, word: &str) -> Result<i32> {
        self.ignored.insert(word.to_string());
        Ok(0)
    }

    fn is_ignored(&self, word: &str) -> Result<bool> {
        Ok(self.ignored.contains(word))
    }

    fn add_extension(&mut self, ext: &str) -> Result<()> {
        self.ignored_for_ext.insert(ext.to_string(), vec![]);
        Ok(())
    }

    fn add_file(&mut self, path: &str) -> Result<()> {
        self.ignored_for_file.insert(path.to_string(), vec![]);
        Ok(())
    }

    fn add_ignored_for_extension(&mut self, word: &str, ext: &str) -> Result<()> {
        let entry = &mut self
            .ignored_for_ext
            .get_mut(ext)
            .ok_or_else(|| anyhow!("Unknown ext: {}", ext))?;
        entry.push(word.to_string());
        Ok(())
    }

    fn add_ignored_for_file(&mut self, word: &str, file: &str) -> Result<()> {
        let entry = self
            .ignored_for_file
            .get_mut(file)
            .ok_or_else(|| anyhow!("Unknown file: {}", file))?;
        entry.push(word.to_string());
        Ok(())
    }

    fn lookup_word(&self, word: &str, path: &Path) -> Result<bool> {
        let full_path = path.to_str();
        let ext = path.extension().and_then(|x| x.to_str());
        let file_name = path.file_name().and_then(|f| f.to_str());

        if self.good.contains(word) {
            return Ok(true);
        }

        if self.ignored.contains(word) {
            return Ok(true);
        }

        if let Some(f) = file_name {
            if self.skipped_file_names.contains(f) {
                return Ok(true);
            }
        }

        if let Some(ext) = ext {
            if let Some(for_ext) = self.ignored_for_ext.get(ext) {
                if for_ext.contains(&word.to_string()) {
                    return Ok(true);
                }
            }
        }

        if let Some(full_path) = full_path {
            if let Some(for_file) = self.ignored_for_file.get(full_path) {
                if for_file.contains(&word.to_string()) {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    fn is_skipped(&self, path: &Path) -> Result<bool> {
        let full_path = path.to_str().unwrap();
        if self.skipped_paths.contains(full_path) {
            return Ok(true);
        }

        let file_name = path.file_name().unwrap().to_str().unwrap();
        if self.skipped_file_names.contains(file_name) {
            return Ok(true);
        }

        Ok(false)
    }

    fn known_extension(&self, ext: &str) -> Result<bool> {
        Ok(self.ignored_for_ext.contains_key(ext))
    }

    fn known_file(&self, full_path: &str) -> Result<bool> {
        Ok(self.ignored_for_file.contains_key(full_path))
    }

    fn skip_file_name(&mut self, file_name: &str) -> Result<()> {
        self.skipped_file_names.insert(file_name.to_string());
        Ok(())
    }

    fn unskip_file_name(&mut self, file_name: &str) -> Result<()> {
        self.skipped_file_names.remove(file_name);
        Ok(())
    }

    fn skip_full_path(&mut self, full_path: &str) -> Result<()> {
        self.skipped_paths.insert(full_path.to_string());
        Ok(())
    }

    fn unskip_full_path(&mut self, full_path: &str) -> Result<()> {
        self.skipped_paths.remove(full_path);
        Ok(())
    }

    fn remove_ignored(&mut self, word: &str) -> Result<()> {
        self.ignored.remove(word);
        Ok(())
    }

    fn remove_ignored_for_extension(&mut self, word: &str, ext: &str) -> Result<()> {
        let entry = &mut self.ignored_for_ext.get_mut(ext);
        if let Some(e) = entry {
            e.retain(|x| x != word);
        }

        Ok(())
    }

    fn remove_ignored_for_file(&mut self, word: &str, path: &str) -> Result<()> {
        let entry = &mut self.ignored_for_file.get_mut(path);
        if let Some(e) = entry {
            e.retain(|x| x != word);
        }
        Ok(())
    }
}
