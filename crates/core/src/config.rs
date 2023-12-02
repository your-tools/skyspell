use std::fmt::Display;
use std::path::{Path, PathBuf};

use anyhow::Context;
use anyhow::{anyhow, bail, Result};

use kdl::{KdlDocument, KdlIdentifier, KdlNode};
use miette::{Diagnostic, SourceSpan};

use crate::RelativePath;

#[derive(Debug, Clone, Copy)]
enum IndentLevel {
    One,
    Two,
}

fn sort_nodes(x: &KdlNode, y: &KdlNode) -> std::cmp::Ordering {
    x.name().value().cmp(y.name().value())
}

#[derive(Debug)]
pub struct Config {
    doc: KdlDocument,
    path: Option<PathBuf>,
}

impl Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.doc)
    }
}

impl Config {
    pub fn open(config_path: &Path) -> Result<Self> {
        if !config_path.exists() {
            std::fs::write(config_path, "").with_context(|| {
                format!("While creating empty config at {}:", config_path.display())
            })?;
        }
        let contents = std::fs::read_to_string(config_path)
            .with_context(|| format!("While reading {}:", config_path.display()))?;
        let doc = Self::parse_doc(&contents)
            .with_context(|| format!("While parsing {}:", config_path.display()))?;
        Ok(Config {
            path: Some(config_path.to_path_buf()),
            doc,
        })
    }

    pub fn new_for_tests() -> Result<Self> {
        let res = Self::parse("").unwrap();
        Ok(res)
    }

    pub fn parse(contents: &str) -> Result<Self> {
        let doc: KdlDocument = Self::parse_doc(contents)?;
        Ok(Config { doc, path: None })
    }

    fn parse_doc(contents: &str) -> Result<KdlDocument> {
        contents.parse::<KdlDocument>().map_err(move |e| {
            let source = e
                .source_code()
                .expect("parse errors should have source code");
            let span: SourceSpan = e.span;
            let contents = source
                .read_span(&span, 0, 0)
                .expect("source should have span contents");
            // miette uses 0 based indexes, but humans prefer 1-based
            let line = contents.line() + 1;
            let column = contents.column() + 1;
            anyhow!("line {line}, column {column}: {e}")
        })
    }

    pub fn provider(&self) -> Option<&str> {
        let entry = self.doc.get("provider")?;
        for entry in entry.entries() {
            if entry.name().map(|i| i.value()) == Some("name") {
                return entry.value().as_string();
            }
        }
        None
    }

    pub fn patterns(&self) -> Vec<&str> {
        let res = match self.doc.get("patterns") {
            Some(x) => x,
            None => return vec![],
        };
        let res = match res.children() {
            Some(x) => x,
            None => return vec![],
        };
        res.nodes().iter().map(|x| x.name().value()).collect()
    }

    pub fn use_db(&self) -> bool {
        self.doc.get("use_db").is_some()
    }

    fn global_words(&self) -> Vec<String> {
        self.words_for_key("global")
    }

    fn global_words_mut(&mut self) -> Option<&mut KdlDocument> {
        self.words_for_key_mut("global")
    }

    fn project_words(&self) -> Vec<String> {
        self.words_for_key("project")
    }

    fn project_words_mut(&mut self) -> Option<&mut KdlDocument> {
        self.words_for_key_mut("project")
    }

    fn ignored_words_for_extension(&self, ext: &str) -> Vec<String> {
        self.words_for_section("extensions", ext)
    }

    fn ignored_words_for_extension_mut(&mut self, ext: &str) -> Result<Option<&mut KdlDocument>> {
        self.words_for_section_mut("extensions", ext)
    }

    fn ignored_words_for_path(&self, path: &str) -> Vec<String> {
        self.words_for_section("paths", path)
    }

    fn ignored_words_for_path_mut(&mut self, path: &str) -> Result<Option<&mut KdlDocument>> {
        self.words_for_section_mut("paths", path)
    }

    fn words_for_key(&self, key: &'static str) -> Vec<String> {
        let section = match self.doc.get(key) {
            None => return vec![],
            Some(s) => s,
        };
        let children = match section.children() {
            None => return vec![],
            Some(c) => c,
        };
        let nodes = children.nodes();
        nodes.iter().map(|x| x.name().value().to_string()).collect()
    }

    fn words_for_key_mut(&mut self, key: &'static str) -> Option<&mut KdlDocument> {
        let node = match self.doc.get_mut(key) {
            None => return None,
            Some(n) => n,
        };
        node.children_mut().as_mut()
    }

    fn words_for_section(&self, key: &'static str, value: &str) -> Vec<String> {
        let section_node = match self.doc.get(key) {
            None => return vec![],
            Some(s) => s,
        };
        let entries = match section_node.children() {
            None => return vec![],
            Some(e) => e,
        };
        for node in entries.nodes() {
            if node.name().value() == value {
                let children = match node.children() {
                    None => return vec![],
                    Some(c) => c,
                };
                let nodes = children.nodes();
                return nodes.iter().map(|x| x.name().value().to_string()).collect();
            }
        }
        vec![]
    }

    fn words_for_section_mut(
        &mut self,
        key: &'static str,
        value: &str,
    ) -> Result<Option<&mut KdlDocument>> {
        let section_node = self
            .doc
            .get_mut(key)
            .ok_or_else(|| anyhow!("key '{key}' should be present"))?;
        let entries = section_node
            .children_mut()
            .as_mut()
            .ok_or_else(|| anyhow!("key '{key}' should have children"))?;
        for node in entries.nodes_mut() {
            if node.name().value() == value {
                let words = node
                    .children_mut()
                    .as_mut()
                    .ok_or_else(|| anyhow!("section '{key}' should have children"))?;
                return Ok(Some(words));
            }
        }
        Ok(None)
    }

    fn make_word_node(word: &str) -> KdlNode {
        let identifier = KdlIdentifier::from(word);
        KdlNode::new(identifier)
    }

    fn add_to_section(&mut self, section: &'static str, word: &str) -> Result<()> {
        let entries = match self.doc.get_mut(section) {
            Some(e) => e,
            None => {
                let new_node = KdlNode::new(KdlIdentifier::from(section));
                self.doc.nodes_mut().push(new_node);
                self.doc.get_mut(section).expect("just created")
            }
        };
        let children = entries.ensure_children();
        let word_node = Self::make_word_node(word);
        Self::insert_word_in_section(word_node, children, IndentLevel::One);
        Ok(())
    }

    fn insert_in_section_with_value(
        &mut self,
        word: &str,
        section: &'static str,
        value: &str,
    ) -> anyhow::Result<()> {
        let section_node = match self.doc.get_mut(section) {
            Some(s) => s,
            None => {
                let new_node = KdlNode::new(KdlIdentifier::from(section));
                self.doc.nodes_mut().push(new_node);
                self.doc.get_mut(section).expect("")
            }
        };
        let entries = section_node.children_mut();

        // Look for a section with a matching name
        let mut matching_node = None;
        for entry in entries {
            for node in entry.nodes_mut() {
                if node.name().value() == value {
                    matching_node = Some(node);
                }
            }
        }

        let node = match matching_node {
            // Not found: create a new section
            None => return self.create_new_section_with(section, value, word),
            Some(n) => n,
        };
        // Found: insert the word in the section
        let word_node = Self::make_word_node(word);
        let doc = node.ensure_children();
        Self::insert_word_in_section(word_node, doc, IndentLevel::Two);
        Ok(())
    }

    fn create_new_section_with(
        &mut self,
        section: &'static str,
        value: &str,
        word: &str,
    ) -> Result<()> {
        let mut words = KdlDocument::new();
        let word_node = Self::make_word_node(word);
        Self::insert_word_in_section(word_node, &mut words, IndentLevel::Two);
        let mut section_node = KdlNode::new(KdlIdentifier::from(value));
        section_node.set_children(words);
        section_node.set_leading("\n  ");
        section_node.set_trailing("\n");

        let parent_node = self
            .doc
            .get_mut(section)
            .ok_or_else(|| anyhow!("section '{section}' should always exist"))?;
        let children = parent_node.ensure_children();
        let nodes = children.nodes_mut();
        nodes.push(section_node);
        Ok(())
    }

    /// Insert a word in a section with a proper indent level
    /// We control everything here: words are sorted and aligned
    /// automatically. The section has no trailing nor leading
    /// whitespace
    fn insert_word_in_section(
        node: KdlNode,
        document: &mut KdlDocument,
        indent_level: IndentLevel,
    ) {
        let (leading_first, leading, trailing, trailing_last) = match indent_level {
            IndentLevel::One => ("\n  ", "  ", "\n", "\n"),
            IndentLevel::Two => ("\n    ", "    ", "\n  ", "\n"),
        };
        let words = document.nodes_mut();
        words.push(node);
        words.sort_by(sort_nodes);
        let last_index = words.len() - 1;
        for (i, word) in words.iter_mut().enumerate() {
            if i == 0 {
                word.set_leading(leading_first);
            } else {
                word.set_leading(leading);
            }
            if i == last_index {
                word.set_trailing(trailing);
            } else {
                word.set_trailing(trailing_last);
            }
        }
        document.set_leading("");
        document.set_trailing("");
    }

    fn save(&self) -> Result<()> {
        let path = match &self.path {
            None => return Ok(()),
            Some(p) => p,
        };
        std::fs::write(path, self.doc.to_string()).with_context(|| "While writing")
    }

    // Should this word be ignored?
    // This is called when a word is *not* found in the spelling dictionary.
    //
    // A word is ignored if:
    //   * it's in the global ignore list
    //   * the relative path has an extension and it's in the ignore list
    //     for this extension
    //   * it's in the ignore list for the project
    //   * it's in the ignore list for the relative path
    //
    // Otherwise, it's *not* ignored and the Checker will call handle_error()
    //
    pub fn should_ignore(&mut self, word: &str, relative_path: &RelativePath) -> Result<bool> {
        if self.is_ignored(word)? {
            return Ok(true);
        }

        if let Some(e) = relative_path.extension() {
            if self.is_ignored_for_extension(word, &e)? {
                return Ok(true);
            }
        }

        if self.is_ignored_for_project(word)? {
            return Ok(true);
        }

        self.is_ignored_for_path(word, relative_path)
    }

    pub fn is_ignored(&mut self, word: &str) -> Result<bool> {
        let global_words = self.global_words();
        Ok(global_words.contains(&word.to_string()))
    }

    pub fn is_ignored_for_extension(&mut self, word: &str, extension: &str) -> Result<bool> {
        Ok(self
            .ignored_words_for_extension(extension)
            .contains(&word.to_string()))
    }

    pub fn is_ignored_for_project(&mut self, word: &str) -> Result<bool> {
        let project_words = self.project_words();
        Ok(project_words.contains(&word.to_string()))
    }

    pub fn is_ignored_for_path(
        &mut self,
        word: &str,
        relative_path: &crate::RelativePath,
    ) -> Result<bool> {
        let for_path = self.ignored_words_for_path(&relative_path.as_str());
        Ok(for_path.contains(&word.to_string()))
    }

    pub fn ignore(&mut self, word: &str) -> Result<()> {
        self.add_to_section("global", word)?;
        self.save()
    }

    pub fn ignore_for_extension(&mut self, word: &str, ext: &str) -> Result<()> {
        self.insert_in_section_with_value(word, "extensions", ext)?;
        self.save()
    }

    pub fn ignore_for_project(&mut self, word: &str) -> Result<()> {
        self.add_to_section("project", word)?;
        self.save()
    }

    pub fn ignore_for_path(
        &mut self,
        word: &str,
        relative_path: &crate::RelativePath,
    ) -> Result<()> {
        self.insert_in_section_with_value(word, "paths", &relative_path.as_str())?;
        self.save()
    }

    pub fn remove_ignored(&mut self, word: &str) -> Result<()> {
        let ignored = match self.global_words_mut() {
            Some(n) => n,
            None => bail!("word was not globally ignored"),
        };
        let nodes = ignored.nodes_mut();
        let before = nodes.len();
        nodes.retain(|x| x.name().value() != word);
        let after = nodes.len();
        if before == after {
            bail!("word was not globally ignored")
        }
        self.save()
    }

    pub fn remove_ignored_for_extension(&mut self, word: &str, extension: &str) -> Result<()> {
        let for_extension = self
            .ignored_words_for_extension_mut(extension)?
            .ok_or_else(|| anyhow!("word was not ignored for this extension"))?;
        let nodes = for_extension.nodes_mut();
        nodes.retain(|x| x.name().value() != word);
        self.save()
    }

    pub fn remove_ignored_for_path(
        &mut self,
        word: &str,
        relative_path: &crate::RelativePath,
    ) -> Result<()> {
        let for_path = self
            .ignored_words_for_path_mut(&relative_path.as_str())?
            .ok_or_else(|| anyhow!("word was not ignored for this path"))?;
        let nodes = for_path.nodes_mut();
        nodes.retain(|x| x.name().value() != word);
        self.save()
    }

    pub fn remove_ignored_for_project(&mut self, word: &str) -> Result<()> {
        let ignored = match self.project_words_mut() {
            Some(i) => i,
            None => return Ok(()),
        };
        let nodes = ignored.nodes_mut();
        nodes.retain(|x| x.name().value() != word);
        self.save()
    }
}

#[cfg(test)]
mod tests;
