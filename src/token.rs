use std::{iter::Peekable, str::CharIndices};

pub struct Tokenizer<'a> {
    input: &'a str,
    iter: Peekable<CharIndices<'a>>,
}

impl<'a> Tokenizer<'a> {
    pub fn new(code: &'a str) -> Self {
        let iter = code.char_indices().peekable();
        Self { input: code, iter }
    }

    fn should_keep(_s: &str) -> bool {
        // TODO: Ignore hexadecimal strings (appearing in hashs, uuid and the like)
        true
    }

    fn read_while<F>(&mut self, start_pos: usize, end_cond: F) -> Option<(&'a str, usize)>
    where
        F: Fn(char) -> bool,
    {
        let end_pos;
        loop {
            let peek = self.iter.peek();
            match peek {
                Some((_, next_c)) if end_cond(*next_c) => {
                    self.iter.next();
                }
                Some((next_pos, _)) => {
                    end_pos = Some(*next_pos);
                    break;
                }
                None => {
                    end_pos = None;
                    break;
                }
            }
        }

        // If end_pos is None, we have reached the end of the input
        let res = match end_pos {
            None => &self.input[start_pos..],
            Some(index) => &self.input[start_pos..index],
        };

        if Self::should_keep(res) {
            Some((res, start_pos))
        } else {
            None
        }
    }

    fn read_lowercase(&mut self, start_pos: usize) -> Option<(&'a str, usize)> {
        self.read_while(start_pos, char::is_lowercase)
    }

    fn read_uppercase(&mut self, start_pos: usize) -> Option<(&'a str, usize)> {
        // Two uppercase: assume SCREAMING_CASE
        // One uppercase followed by a lower case: assume CamelCase
        let (_, next_c) = self.iter.peek()?;
        match next_c {
            c if c.is_uppercase() => self.read_screaming_case(start_pos),
            c if c.is_lowercase() => self.read_camel_case(start_pos),
            _ => None,
        }
    }

    fn read_screaming_case(&mut self, start_pos: usize) -> Option<(&'a str, usize)> {
        self.read_while(start_pos, char::is_uppercase)
    }

    fn read_camel_case(&mut self, start_pos: usize) -> Option<(&'a str, usize)> {
        self.read_while(start_pos, char::is_lowercase)
    }
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = (&'a str, usize);

    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        loop {
            let (pos, c) = self.iter.next()?;
            match c {
                c if c.is_lowercase() => {
                    let next_word = self.read_lowercase(pos);
                    match next_word {
                        Some(w) => return Some(w),
                        _ => continue,
                    }
                }
                c if c.is_uppercase() => {
                    let next_word = self.read_uppercase(pos);
                    match next_word {
                        Some(w) => return Some(w),
                        _ => continue,
                    }
                }
                _ => {
                    continue;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_split_words() {
        let text = "hello world";
        let tokenizer = Tokenizer::new(&text);
        let actual: Vec<_> = tokenizer.map(|(x, _index)| x).collect();
        assert_eq!(&actual, &["hello", "world"]);
    }

    #[test]
    fn test_split_underscore() {
        let text = "foo_bar = spam()";
        let tokenizer = Tokenizer::new(&text);
        let actual: Vec<_> = tokenizer.map(|(x, _index)| x).collect();
        assert_eq!(&actual, &["foo", "bar", "spam"]);
    }

    #[test]
    fn test_split_camel_case() {
        let text = "FooBar spamEggs = SCREAMING_CONSTANT";
        let tokenizer = Tokenizer::new(&text);
        let actual: Vec<_> = tokenizer.map(|(x, _index)| x).collect();
        assert_eq!(
            &actual,
            &["Foo", "Bar", "spam", "Eggs", "SCREAMING", "CONSTANT"]
        );
    }
}
