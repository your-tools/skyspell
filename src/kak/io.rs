use anyhow::{anyhow, Context, Result};

use std::path::PathBuf;

use crate::kak::checker::{SKYSPELL_LANG_OPT, SKYSPELL_PROJECT_OPT};
use crate::Project;

pub(crate) trait OperatingSystemIO {
    fn get_env_var(&self, key: &str) -> Result<String>;
    fn print(&self, text: &str);
}

pub(crate) struct KakouneIO<S: OperatingSystemIO> {
    os_io: S,
}

impl<S: OperatingSystemIO> KakouneIO<S> {
    pub(crate) fn new(os_io: S) -> Self {
        Self { os_io }
    }

    #[allow(dead_code)]
    pub(crate) fn debug(&self, message: &str) {
        self.os_io.print(&format!("echo -debug {}", message));
    }

    pub(crate) fn get_variable(&self, key: &str) -> Result<String> {
        self.os_io.get_env_var(key)
    }

    pub(crate) fn get_option(&self, name: &str) -> Result<String> {
        let key = format!("kak_opt_{}", name);
        self.os_io.get_env_var(&key)
    }

    pub(crate) fn parse_usize(&self, v: &str) -> Result<usize> {
        v.parse()
            .map_err(|_| anyhow!("could not parse {} as a positive number"))
    }

    pub(crate) fn get_cursor(&self) -> Result<(usize, usize)> {
        let line = self.get_variable("kak_cursor_line")?;
        let column = self.get_variable("kak_cursor_column")?;
        Ok((self.parse_usize(&line)?, self.parse_usize(&column)?))
    }

    pub(crate) fn get_selection(&self) -> Result<String> {
        self.get_variable("kak_selection")
    }

    pub(crate) fn get_project(&self) -> Result<Project> {
        let as_str = self.get_option(SKYSPELL_PROJECT_OPT)?;
        let path = PathBuf::from(as_str);
        Project::new(&path)
    }

    pub(crate) fn goto_previous_buffer(&self) {
        self.os_io.print("execute-keys ga\n")
    }

    pub(crate) fn parse_cursor(&self, pos: &str) -> Result<(usize, usize)> {
        let (start, end) = pos.split_once('.').context("cursor should contain '.'")?;
        let start = start
            .parse::<usize>()
            .context("could not parse cursor start as an integer")?;
        let end = end
            .parse::<usize>()
            .context("could not parse cursor end as an integer")?;
        Ok((start, end))
    }

    pub(crate) fn parse_range_spec(&self, range_spec: &str) -> Result<Vec<(usize, usize, usize)>> {
        // range-spec is empty
        if range_spec == "0" {
            return Ok(vec![]);
        }

        // Skip the timestamp
        let mut split = range_spec.split_whitespace();
        split.next();

        split.into_iter().map(|x| self.parse_range(x)).collect()
    }

    fn parse_range(&self, range: &str) -> Result<(usize, usize, usize)> {
        let (range, _face) = range
            .split_once('|')
            .context("range spec should contain a face")?;
        let (start, end) = range
            .split_once(',')
            .context("range spec should contain ','")?;

        let (start_line, start_col) = self.parse_cursor(start)?;
        let (_end_line, end_col) = self.parse_cursor(end)?;

        Ok((start_line, start_col, end_col))
    }

    pub(crate) fn get_previous_selection<'a>(
        &self,
        cursor: (usize, usize),
        ranges: &'a [(usize, usize, usize)],
    ) -> Option<&'a (usize, usize, usize)> {
        let (cursor_line, cursor_col) = cursor;
        for range in ranges.iter().rev() {
            let &(start_line, _start_col, end_col) = range;
            if start_line > cursor_line {
                continue;
            }

            if start_line == cursor_line && end_col >= cursor_col {
                continue;
            }
            return Some(range);
        }

        // If we reach there, return the last error (auto-wrap)
        ranges.iter().last()
    }

    pub(crate) fn get_next_selection<'a>(
        &self,
        cursor: (usize, usize),
        ranges: &'a [(usize, usize, usize)],
    ) -> Option<&'a (usize, usize, usize)> {
        let (cursor_line, cursor_col) = cursor;
        for range in ranges.iter() {
            let &(start_line, _start_col, end_col) = range;

            if start_line < cursor_line {
                continue;
            }

            if start_line == cursor_line && end_col <= cursor_col {
                continue;
            }
            return Some(range);
        }

        // If we reach there, return the first error (auto-wrap)
        ranges.iter().next()
    }

    pub(crate) fn get_lang(&self) -> Result<String> {
        self.get_option(SKYSPELL_LANG_OPT)
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::collections::HashMap;

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

    type FakeKakouneIO = KakouneIO<FakeIO>;

    impl FakeKakouneIO {
        fn get_output(&self) -> String {
            self.os_io.stdout.borrow().to_string()
        }
    }

    fn new_fake_io() -> FakeKakouneIO {
        let interactor = FakeIO::new();
        KakouneIO::new(interactor)
    }

    #[test]
    fn test_get_previous_selection() {
        let kakoune_io = new_fake_io();
        let pos = (1, 21);
        let ranges = [(1, 12, 19), (2, 19, 27)];
        let actual = kakoune_io.get_previous_selection(pos, &ranges).unwrap();
        assert_eq!(actual, &(1, 12, 19));
    }

    #[test]
    fn test_get_selection() {
        let kakoune_io = new_fake_io();
        let err = kakoune_io.get_selection().unwrap_err();
        assert_eq!(err.to_string(), "No such key: kak_selection");
    }

    #[test]
    fn test_goto_previous_buffer() {
        let kakoune_io = new_fake_io();
        kakoune_io.goto_previous_buffer();
        assert_eq!(kakoune_io.get_output(), "execute-keys ga\n");
    }
}
