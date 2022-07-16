use std::fmt::Display;

use anyhow::{anyhow, bail};
use textwrap;

use kdl::{KdlDocument, KdlIdentifier, KdlNode};

use crate::IgnoreStore;
use crate::ProjectId;

const SECTIONS: [&str; 4] = ["global", "project", "extensions", "paths"];
// We need a project_id because it's found in the arguments of some
// methods of the trait, but we never use its value
const MAGIC_PROJECT_ID: ProjectId = 42;

#[derive(Debug, Clone, Copy)]
enum IndentLevel {
    One,
    Two,
}

fn sort_nodes(x: &KdlNode, y: &KdlNode) -> std::cmp::Ordering {
    x.name().value().cmp(y.name().value())
}

#[derive(Debug)]
pub struct IgnoreConfig {
    doc: KdlDocument,
}

impl Default for IgnoreConfig {
    fn default() -> Self {
        let input = r#"
        global {
            
        }
        
        project {
            
        }
        
        extensions {
            
        }
         
        paths {
            
        }
        "#;
        let input = textwrap::dedent(input);
        let doc: KdlDocument = input.parse().expect("hard-coded config should be valid");
        Self { doc }
    }
}

impl IgnoreConfig {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn new_for_tests() -> anyhow::Result<Self> {
        Ok(Self::new())
    }

    pub fn parse(kdl: &str) -> Result<Self, String> {
        let doc = match kdl.parse::<KdlDocument>() {
            Ok(doc) => doc,
            Err(e) => return Err(e.to_string()),
        };
        for section in SECTIONS {
            if doc.get(section).is_none() {
                return Err(format!("Missing '{section}' section"));
            }
        }
        Ok(IgnoreConfig { doc })
    }

    fn global_words(&self) -> &KdlDocument {
        self.words_for_key("global")
    }

    fn global_words_mut(&mut self) -> &mut KdlDocument {
        self.words_for_key_mut("global")
    }

    fn project_words(&self) -> &KdlDocument {
        self.words_for_key("project")
    }

    fn project_words_mut(&mut self) -> &mut KdlDocument {
        self.words_for_key_mut("project")
    }

    fn ignored_words_for_extension(&self, ext: &str) -> Option<&KdlDocument> {
        self.words_for_section("extensions", ext)
    }

    fn ignored_words_for_extension_mut(&mut self, ext: &str) -> Option<&mut KdlDocument> {
        self.words_for_section_mut("extensions", ext)
    }

    fn ignored_words_for_path(&self, path: &str) -> Option<&KdlDocument> {
        self.words_for_section("paths", path)
    }

    fn ignored_words_for_path_mut(&mut self, path: &str) -> Option<&mut KdlDocument> {
        self.words_for_section_mut("paths", path)
    }

    fn words_for_key(&self, key: &'static str) -> &KdlDocument {
        self.doc
            .get(key)
            .expect("key '{key}' should exist")
            .children()
            .expect("key '{key}' should have children")
    }

    fn words_for_key_mut(&mut self, key: &'static str) -> &mut KdlDocument {
        self.doc
            .get_mut(key)
            .expect("key '{key}' should exist")
            .children_mut()
            .as_mut()
            .expect("key '{key}' should have children")
    }

    fn words_for_section(&self, key: &'static str, value: &str) -> Option<&KdlDocument> {
        let extensions = self.doc.get(key).expect("section '{key}' should exist");
        let entries = extensions
            .children()
            .expect("section '{key} should have children");
        for node in entries.nodes() {
            if node.name().value() == value {
                let words = node
                    .children()
                    .expect("section '{key}' should have children");
                return Some(words);
            }
        }
        None
    }

    fn words_for_section_mut(
        &mut self,
        key: &'static str,
        value: &str,
    ) -> Option<&mut KdlDocument> {
        let extensions = self.doc.get_mut(key).expect("section '{key}' should exist");
        let entries = extensions.children_mut();
        for entry in entries {
            for node in entry.nodes_mut() {
                if node.name().value() == value {
                    let words = node
                        .children_mut()
                        .as_mut()
                        .expect("section '{key}' should have children");
                    return Some(words);
                }
            }
        }
        None
    }

    fn make_word_node(word: &str) -> KdlNode {
        let identifier = KdlIdentifier::from(word);
        KdlNode::new(identifier)
    }

    fn add_to_section(&mut self, section: &'static str, word: &str) {
        let entries = self.doc.get_mut(section).expect("section should exist");
        let children = entries.ensure_children();
        let word_node = Self::make_word_node(word);
        Self::insert_word_in_section(word_node, children, IndentLevel::One);
    }

    fn insert_in_section_with_value(
        &mut self,
        word: &str,
        section: &'static str,
        value: &str,
    ) -> anyhow::Result<()> {
        let mut matching_node = None;
        let section_node = self
            .doc
            .get_mut(section)
            .expect("section '{section}' should exist");
        let entries = section_node.children_mut();
        // Look for a section with a matching name
        for entry in entries {
            for node in entry.nodes_mut() {
                if node.name().value() == value {
                    matching_node = Some(node);
                }
            }
        }

        let node = match matching_node {
            // Not found: create a new section
            None => {
                return {
                    self.create_new_section_with(section, value, word);
                    Ok(())
                }
            }
            Some(n) => n,
        };
        // Found: insert the word in the section
        let word_node = Self::make_word_node(word);
        let doc = node.ensure_children();
        Self::insert_word_in_section(word_node, doc, IndentLevel::Two);
        Ok(())
    }

    fn create_new_section_with(&mut self, section: &'static str, value: &str, word: &str) {
        let mut words = KdlDocument::new();
        let word_node = Self::make_word_node(word);
        Self::insert_word_in_section(word_node, &mut words, IndentLevel::Two);
        let mut section_node = KdlNode::new(KdlIdentifier::from(value));
        section_node.set_children(words);
        section_node.set_leading("\n  ");
        section_node.set_trailing("");

        let parent_node = self
            .doc
            .get_mut(section)
            .expect("section '{section}' should always exist");
        let children = parent_node.ensure_children();
        let nodes = children.nodes_mut();
        nodes.push(section_node);
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
}

impl Display for IgnoreConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.doc)
    }
}

impl IgnoreStore for IgnoreConfig {
    fn is_ignored(&self, word: &str) -> anyhow::Result<bool> {
        let global_words = self.global_words();
        Ok(global_words.get(word).is_some())
    }

    fn is_ignored_for_extension(&self, word: &str, extension: &str) -> anyhow::Result<bool> {
        let for_extension = match self.ignored_words_for_extension(extension) {
            None => return Ok(false),
            Some(e) => e,
        };
        Ok(for_extension.get(word).is_some())
    }

    fn is_ignored_for_project(
        &self,
        word: &str,
        project_id: crate::ProjectId,
    ) -> anyhow::Result<bool> {
        if project_id != MAGIC_PROJECT_ID {
            return Ok(false);
        }
        let project_words = self.project_words();
        Ok(project_words.get(word).is_some())
    }

    fn is_ignored_for_path(
        &self,
        word: &str,
        project_id: crate::ProjectId,
        relative_path: &crate::RelativePath,
    ) -> anyhow::Result<bool> {
        if project_id != MAGIC_PROJECT_ID {
            return Ok(false);
        }
        let for_path = match self.ignored_words_for_path(&relative_path.as_str()) {
            None => return Ok(false),
            Some(e) => e,
        };
        Ok(for_path.get(word).is_some())
    }

    fn insert_ignored_words(&mut self, words: &[&str]) -> anyhow::Result<()> {
        for word in words {
            self.add_to_section("project", word)
        }
        Ok(())
    }

    fn ignore(&mut self, word: &str) -> anyhow::Result<()> {
        self.add_to_section("global", word);
        Ok(())
    }

    fn ignore_for_extension(&mut self, word: &str, ext: &str) -> anyhow::Result<()> {
        self.insert_in_section_with_value(word, "extensions", ext)
    }

    fn ignore_for_project(
        &mut self,
        word: &str,
        _project_id: crate::ProjectId,
    ) -> anyhow::Result<()> {
        self.add_to_section("project", word);
        Ok(())
    }

    fn ignore_for_path(
        &mut self,
        word: &str,
        project_id: crate::ProjectId,
        relative_path: &crate::RelativePath,
    ) -> anyhow::Result<()> {
        if project_id != MAGIC_PROJECT_ID {
            bail!("Should have called with MAGIC_PROJECT_ID");
        }
        self.insert_in_section_with_value(word, "paths", &relative_path.as_str())?;
        println!("{}", self.doc);
        Ok(())
    }

    fn remove_ignored(&mut self, word: &str) -> anyhow::Result<()> {
        let ignored = self.global_words_mut();
        let nodes = ignored.nodes_mut();
        let before = nodes.len();
        nodes.retain(|x| x.name().value() != word);
        let after = nodes.len();
        if before == after {
            bail!("word was not globally ignored")
        }
        Ok(())
    }

    fn remove_ignored_for_extension(&mut self, word: &str, extension: &str) -> anyhow::Result<()> {
        let for_extension = self
            .ignored_words_for_extension_mut(extension)
            .ok_or_else(|| anyhow!("word was not ignored for this extension"))?;
        let nodes = for_extension.nodes_mut();
        nodes.retain(|x| x.name().value() != word);
        Ok(())
    }

    fn remove_ignored_for_path(
        &mut self,
        word: &str,
        project_id: crate::ProjectId,
        relative_path: &crate::RelativePath,
    ) -> anyhow::Result<()> {
        if project_id != MAGIC_PROJECT_ID {
            bail!("Should have called with MAGIC_PROJECT_ID");
        }

        let for_path = self
            .ignored_words_for_path_mut(&relative_path.as_str())
            .ok_or_else(|| anyhow!("word was not ignored for this path"))?;
        let nodes = for_path.nodes_mut();
        nodes.retain(|x| x.name().value() != word);
        Ok(())
    }

    fn remove_ignored_for_project(
        &mut self,
        word: &str,
        project_id: crate::ProjectId,
    ) -> anyhow::Result<()> {
        if project_id != MAGIC_PROJECT_ID {
            bail!("Should have called with MAGIC_PROJECT_ID");
        }
        let ignored = self.project_words_mut();
        let nodes = ignored.nodes_mut();
        nodes.retain(|x| x.name().value() != word);
        Ok(())
    }
}

#[cfg(test)]
mod tests;
