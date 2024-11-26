use std::io::{BufRead, Lines};

use crate::tokens::ExtractMode;

fn get_tokens(line: &str, _extract_mode: ExtractMode) -> Vec<String> {
    let words = line
        .split_ascii_whitespace()
        .map(|x| x.to_owned())
        .collect();
    words
}

pub struct Tokens<B> {
    lines: Lines<B>,
    extract_mode: ExtractMode,
    line_number: usize,
    current_line: String,
    tokens: Vec<String>,
    token_index: usize,
    new_line: bool,
}

impl<B> Tokens<B> {
    pub fn new(lines: Lines<B>, extract_mode: ExtractMode) -> Self {
        Self {
            lines,
            extract_mode,
            line_number: 0,
            tokens: vec![],
            token_index: 0,
            current_line: String::new(),
            new_line: true,
        }
    }
}

impl<B: BufRead> Iterator for Tokens<B> {
    type Item = Result<String, std::io::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.new_line {
                let next_line = self.lines.next();
                let line_result = match next_line {
                    None => return None,
                    Some(l) => l,
                };
                let line = match line_result {
                    Ok(l) => l,
                    Err(e) => return Some(Err(e)),
                };
                self.current_line = line;
                self.tokens = get_tokens(&self.current_line, self.extract_mode);
                self.line_number += 1;
            }

            let token = self.tokens.get(self.token_index);
            self.token_index += 1;
            match token {
                Some(t) => {
                    self.new_line = false;
                    return Some(Ok(t.to_owned()));
                }
                None => {
                    self.new_line = true;
                    self.token_index = 0;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests;
