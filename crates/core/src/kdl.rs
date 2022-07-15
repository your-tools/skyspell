use std::fmt::Display;

use kdl::{KdlDocument, KdlIdentifier, KdlNode};

use crate::IgnoreStore;
use crate::ProjectId;

const SECTIONS: [&str; 3] = ["global", "project", "extensions"];
// We need a project_id because it's found in the arguments of some
// methods of the trait, but we never use its value
const PROJECT_ID: ProjectId = 42;

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

impl IgnoreConfig {
    pub fn new() -> Self {
        let input = r#"
        global {
            
        }
        
        project {
            
        }
        
        extensions {
            
        }
        "#;
        let doc: KdlDocument = input.parse().expect("hard-coded config should be valid");
        Self { doc }
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
        self.doc
            .get("global")
            .expect("global should exist")
            .children()
            .expect("global should have children")
    }

    fn project_words(&self) -> &KdlDocument {
        self.doc
            .get("project")
            .expect("project should exist")
            .children()
            .expect("project should have children")
    }

    fn ignored_words_for_extension(&self, ext: &str) -> Option<&KdlDocument> {
        let extensions = self.doc.get("extensions").expect("extensions should exist");
        let entries = extensions.children();
        for entry in entries {
            for node in entry.nodes() {
                if node.name().value() == ext {
                    let words = node
                        .children()
                        .expect("extensions sections should have children");
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

    fn create_new_extension_section_with(&mut self, word: &str, ext: &str) {
        let mut words = KdlDocument::new();
        let word_node = Self::make_word_node(word);
        Self::insert_word_in_section(word_node, &mut words, IndentLevel::Two);
        let mut extension_node = KdlNode::new(KdlIdentifier::from(ext));
        extension_node.set_children(words);
        extension_node.set_leading("\n  ");
        extension_node.set_trailing("");

        let extensions = self
            .doc
            .get_mut("extensions")
            .expect("extensions should always exist");
        let children = extensions.ensure_children();
        let nodes = children.nodes_mut();
        nodes.push(extension_node);
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
        _project_id: crate::ProjectId,
    ) -> anyhow::Result<bool> {
        let project_words = self.project_words();
        Ok(project_words.get(word).is_some())
    }

    fn is_ignored_for_path(
        &self,
        word: &str,
        _project_id: crate::ProjectId,
        relative_path: &crate::RelativePath,
    ) -> anyhow::Result<bool> {
        todo!()
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

    fn new_project(
        &mut self,
        project_path: &crate::ProjectPath,
    ) -> anyhow::Result<crate::ProjectId> {
        Ok(PROJECT_ID)
    }

    fn project_exists(&self, _project_path: &crate::ProjectPath) -> anyhow::Result<bool> {
        Ok(true)
    }

    fn remove_project(&mut self, _project_id: crate::ProjectId) -> anyhow::Result<()> {
        Ok(())
    }

    fn get_project_id(
        &self,
        _project_path: &crate::ProjectPath,
    ) -> anyhow::Result<crate::ProjectId> {
        Ok(PROJECT_ID)
    }

    fn projects(&self) -> anyhow::Result<Vec<crate::ProjectInfo>> {
        Ok(vec![])
    }

    fn ignore_for_extension(&mut self, word: &str, ext: &str) -> anyhow::Result<()> {
        let mut extension_node = None;
        let extensions = self
            .doc
            .get_mut("extensions")
            .expect("extensions should exist");
        let entries = extensions.children_mut();
        // Look for a section with a matching name
        for entry in entries {
            for node in entry.nodes_mut() {
                if node.name().value() == ext {
                    extension_node = Some(node);
                }
            }
        }

        let node = match extension_node {
            // Not found: create a new section
            None => return Ok(self.create_new_extension_section_with(word, ext)),
            Some(n) => n,
        };
        // Found: insert the word in the section
        let word_node = Self::make_word_node(word);
        let doc = node.ensure_children();
        Self::insert_word_in_section(word_node, doc, IndentLevel::Two);
        Ok(())
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
        todo!()
    }

    fn remove_ignored(&mut self, word: &str) -> anyhow::Result<()> {
        todo!()
    }

    fn remove_ignored_for_extension(&mut self, word: &str, extension: &str) -> anyhow::Result<()> {
        todo!()
    }

    fn remove_ignored_for_path(
        &mut self,
        word: &str,
        project_id: crate::ProjectId,
        relative_path: &crate::RelativePath,
    ) -> anyhow::Result<()> {
        todo!()
    }

    fn remove_ignored_for_project(
        &mut self,
        word: &str,
        project_id: crate::ProjectId,
    ) -> anyhow::Result<()> {
        todo!()
    }

    fn insert_operation(&mut self, _operation: &crate::Operation) -> anyhow::Result<()> {
        Ok(())
    }

    fn pop_last_operation(&mut self) -> anyhow::Result<Option<crate::Operation>> {
        Ok(None)
    }
}

#[cfg(test)]
mod tests;
