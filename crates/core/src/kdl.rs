use std::fmt::Display;

use kdl::{KdlDocument, KdlIdentifier, KdlNode};

const SECTIONS: [&str; 3] = ["global", "project", "extensions"];

fn sort_nodes(x: &KdlNode, y: &KdlNode) -> std::cmp::Ordering {
    x.name().value().cmp(y.name().value())
}

#[derive(Debug)]
pub struct IgnoreConfig {
    doc: KdlDocument,
}

#[derive(Debug, Clone, Copy)]
enum IndentLevel {
    One,
    Two,
}

impl Display for IgnoreConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.doc)
    }
}

impl IgnoreConfig {
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

    pub fn add_global(&mut self, word: &str) {
        self.add_to_section("global", word)
    }

    pub fn add_project(&mut self, word: &str) {
        self.add_to_section("project", word)
    }

    fn add_to_section(&mut self, section: &'static str, word: &str) {
        let entries = self.doc.get_mut(section).unwrap();
        let children = entries.children_mut().as_mut().unwrap();
        let word_node = Self::make_word_node(word);
        Self::insert_word_in_section(word_node, children, IndentLevel::One);
    }

    pub fn add_ignore_for_extension(&mut self, word: &str, ext: &str) {
        let mut extension_node = None;
        let extensions = self.doc.get_mut("extensions").unwrap();
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
            None => return self.create_new_extension_section_with(word, ext),
            Some(n) => n,
        };
        // Found: insert the word in the section
        let word_node = Self::make_word_node(word);
        let doc = node.children_mut().as_mut().unwrap();
        Self::insert_word_in_section(word_node, doc, IndentLevel::Two);
    }

    fn create_new_extension_section_with(&mut self, word: &str, ext: &str) {
        let mut words = KdlDocument::new();
        let word_node = Self::make_word_node(word);
        Self::insert_word_in_section(word_node, &mut words, IndentLevel::Two);
        let mut extension_node = KdlNode::new(KdlIdentifier::from(ext));
        extension_node.set_children(words);
        extension_node.set_leading("\n  ");
        extension_node.set_trailing("");

        let extensions = self.doc.get_mut("extensions").unwrap();
        let children = extensions.children_mut().as_mut().unwrap();
        let nodes = children.nodes_mut();
        nodes.push(extension_node);
    }

    fn make_word_node(word: &str) -> KdlNode {
        let identifier = KdlIdentifier::from(word);
        KdlNode::new(identifier)
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

#[cfg(test)]
mod tests;
