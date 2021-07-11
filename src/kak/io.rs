use anyhow::{anyhow, Context, Result};

pub trait OperatingSystemIO {
    fn get_env_var(&self, key: &str) -> Result<String>;
    fn print(&self, text: &str);
}

pub struct KakouneIO<S: OperatingSystemIO> {
    os_io: S,
}

#[derive(Copy, Clone)]
pub struct StandardIO;

impl OperatingSystemIO for StandardIO {
    fn get_env_var(&self, key: &str) -> Result<String> {
        std::env::var(key).map_err(|_| anyhow!("{} not found in environment", key))
    }

    fn print(&self, text: &str) {
        print!("{}", text);
    }
}

pub type StdKakouneIO = KakouneIO<StandardIO>;

pub fn new_kakoune_io() -> StdKakouneIO {
    let io = StandardIO;
    KakouneIO::new(io)
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
            .map_err(|_| anyhow!("could not parse '{}' as a positive number", v))
    }

    pub(crate) fn get_cursor(&self) -> Result<(usize, usize)> {
        let line = self.get_variable("kak_cursor_line")?;
        let column = self.get_variable("kak_cursor_column")?;
        Ok((self.parse_usize(&line)?, self.parse_usize(&column)?))
    }

    pub(crate) fn get_selection(&self) -> Result<String> {
        self.get_variable("kak_selection")
    }

    pub(crate) fn get_timestamp(&self) -> Result<usize> {
        let timestamp = self.os_io.get_env_var("kak_timestamp")?;
        self.parse_usize(&timestamp)
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

    pub(crate) fn print(&self, command: &str) {
        self.os_io.print(command);
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
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::collections::HashMap;

    pub(crate) struct FakeOperatingSystemIO {
        env: HashMap<String, String>,
        stdout: RefCell<String>,
    }

    impl FakeOperatingSystemIO {
        pub(crate) fn new() -> Self {
            Self {
                env: HashMap::new(),
                stdout: RefCell::new(String::new()),
            }
        }
    }

    impl OperatingSystemIO for FakeOperatingSystemIO {
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

    pub(crate) type FakeKakouneIO = KakouneIO<FakeOperatingSystemIO>;

    impl FakeKakouneIO {
        pub(crate) fn get_output(&self) -> String {
            self.os_io.stdout.borrow().to_string()
        }

        pub(crate) fn set_env_var(&mut self, key: &str, value: &str) {
            self.os_io.env.insert(key.to_string(), value.to_string());
        }

        pub(crate) fn set_option(&mut self, key: &str, value: &str) {
            let key = format!("kak_opt_{}", key);
            self.os_io.env.insert(key, value.to_string());
        }

        pub(crate) fn set_selection(&mut self, text: &str) {
            self.set_env_var("kak_selection", text)
        }

        pub(crate) fn set_timestamp(&mut self, timestamp: usize) {
            self.set_env_var("kak_timestamp", &timestamp.to_string())
        }

        pub(crate) fn set_cursor(&mut self, line: usize, column: usize) {
            self.set_env_var("kak_cursor_line", &line.to_string());
            self.set_env_var("kak_cursor_column", &column.to_string());
        }
    }

    pub(crate) fn new_fake_io() -> FakeKakouneIO {
        let fake_os_io = FakeOperatingSystemIO::new();
        KakouneIO::new(fake_os_io)
    }

    #[test]
    fn test_debug() {
        let kakoune_io = new_fake_io();
        kakoune_io.debug("This is a debug message");
        let actual = kakoune_io.get_output();
        assert_eq!(actual, "echo -debug This is a debug message");
    }

    #[test]
    fn test_get_variable_no_such_key() {
        let kakoune_io = new_fake_io();
        let actual = kakoune_io.get_variable("no-such-key");
        assert!(actual.is_err());
    }

    #[test]
    fn test_get_variable_set_in_fake_io() {
        let mut kakoune_io = new_fake_io();
        kakoune_io.set_env_var("my_key", "my_value");
        let actual = kakoune_io.get_variable("my_key").unwrap();
        assert_eq!(actual, "my_value");
    }

    #[test]
    fn test_get_option_no_such_key() {
        let kakoune_io = new_fake_io();
        let actual = kakoune_io.get_option("no-such-key");
        assert!(actual.is_err());
    }

    #[test]
    fn test_get_option_set_in_fake_io() {
        let mut kakoune_io = new_fake_io();
        kakoune_io.set_option("my_opt", "my_value");
        let actual = kakoune_io.get_option("my_opt").unwrap();
        assert_eq!(actual, "my_value");
    }

    #[test]
    fn test_get_timestamp_not_set() {
        let kakoune_io = new_fake_io();
        let actual = kakoune_io.get_timestamp();
        assert!(actual.is_err());
    }

    #[test]
    fn test_get_timestamp_set_in_fake_io() {
        let mut kakoune_io = new_fake_io();
        kakoune_io.set_timestamp(42);
        let actual = kakoune_io.get_timestamp().unwrap();
        assert_eq!(actual, 42);
    }

    #[test]
    fn test_parse_usize_happy() {
        let kakoune_io = new_fake_io();
        let actual = kakoune_io.parse_usize("42").unwrap();
        assert_eq!(actual, 42usize);
    }

    #[test]
    fn test_parse_usize_invalid() {
        let kakoune_io = new_fake_io();
        let actual = kakoune_io.parse_usize("-3");
        assert!(actual.is_err());
    }

    #[test]
    fn test_get_cursor_set_in_fake() {
        let mut kakoune_io = new_fake_io();
        kakoune_io.set_cursor(3, 45);
        let actual = kakoune_io.get_cursor().unwrap();
        assert_eq!(actual, (3, 45));
    }

    #[test]
    fn test_parse_range_spec_empty() {
        let kakoune_io = new_fake_io();
        let actual = kakoune_io.parse_range_spec("0").unwrap();
        assert!(actual.is_empty());
    }

    #[test]
    fn test_parse_range_spec_not_empty() {
        let kakoune_io = new_fake_io();
        let actual = kakoune_io
            .parse_range_spec("42 1.4,1.13|Error 1.15,1.23|Error")
            .unwrap();
        assert_eq!(actual, vec![(1, 4, 13), (1, 15, 23)]);
    }

    #[test]
    fn test_get_selection_missing_key() {
        let kakoune_io = new_fake_io();
        let err = kakoune_io.get_selection().unwrap_err();
        assert_eq!(err.to_string(), "No such key: kak_selection");
    }

    #[test]
    fn test_get_selection_set_in_fake() {
        let mut kakoune_io = new_fake_io();
        kakoune_io.set_selection("selected text");
        let actual = kakoune_io.get_selection().unwrap();
        assert_eq!(actual, "selected text");
    }

    #[test]
    fn test_goto_previous_buffer() {
        let kakoune_io = new_fake_io();
        kakoune_io.goto_previous_buffer();
        assert_eq!(kakoune_io.get_output(), "execute-keys ga\n");
    }

    #[test]
    fn test_get_previous_selection_between_two_selections_other_line() {
        let kakoune_io = new_fake_io();
        let pos = (1, 21);
        let ranges = [(1, 12, 19), (2, 19, 27)];
        let actual = kakoune_io.get_previous_selection(pos, &ranges).unwrap();
        assert_eq!(actual, &(1, 12, 19));
    }

    #[test]
    fn test_get_previous_selection_from_next_line() {
        let kakoune_io = new_fake_io();
        let pos = (2, 1);
        let ranges = [(1, 12, 19)];
        let actual = kakoune_io.get_previous_selection(pos, &ranges).unwrap();
        assert_eq!(actual, &(1, 12, 19));
    }

    #[test]
    fn test_get_previous_selection_wraps() {
        let kakoune_io = new_fake_io();
        let pos = (1, 1);
        let ranges = [(2, 3, 5), (3, 12, 19)];
        let actual = kakoune_io.get_previous_selection(pos, &ranges).unwrap();
        assert_eq!(actual, &(3, 12, 19));
    }

    #[test]
    fn test_get_previous_selection_between_two_selections_same_line() {
        let kakoune_io = new_fake_io();
        let pos = (1, 15);
        let ranges = [(1, 12, 13), (1, 19, 21)];
        let actual = kakoune_io.get_previous_selection(pos, &ranges).unwrap();
        assert_eq!(actual, &(1, 12, 13));
    }

    #[test]
    fn test_get_next_selection_from_previous_line() {
        let kakoune_io = new_fake_io();
        let pos = (1, 1);
        let ranges = [(2, 12, 19)];
        let actual = kakoune_io.get_next_selection(pos, &ranges).unwrap();
        assert_eq!(actual, &(2, 12, 19));
    }

    #[test]
    fn test_get_next_selection_wraps() {
        let kakoune_io = new_fake_io();
        let pos = (4, 1);
        let ranges = [(2, 3, 5), (3, 12, 19)];
        let actual = kakoune_io.get_next_selection(pos, &ranges).unwrap();
        assert_eq!(actual, &(2, 3, 5));
    }

    #[test]
    fn test_get_next_selection_between_two_selections_same_line() {
        let kakoune_io = new_fake_io();
        let pos = (1, 15);
        let ranges = [(1, 12, 13), (1, 19, 21)];
        let actual = kakoune_io.get_next_selection(pos, &ranges).unwrap();
        assert_eq!(actual, &(1, 19, 21));
    }
}
