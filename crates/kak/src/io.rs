use anyhow::{anyhow, Context, Result};
use skyspell_core::{OperatingSystemIO, StandardIO};

pub struct KakouneIO<S: OperatingSystemIO> {
    os_io: S,
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
        self.os_io.print(&format!("echo -debug {}\n", message));
    }

    pub fn get_variable(&self, key: &str) -> Result<String> {
        self.os_io.get_env_var(key)
    }

    pub fn get_option(&self, name: &str) -> Result<String> {
        let key = format!("kak_opt_{}", name);
        self.os_io.get_env_var(&key)
    }

    pub(crate) fn parse_usize(&self, v: &str) -> Result<usize> {
        v.parse()
            .map_err(|_| anyhow!("could not parse '{}' as a positive number", v))
    }

    pub fn get_cursor(&self) -> Result<(usize, usize)> {
        let line = self.get_variable("kak_cursor_line")?;
        let column = self.get_variable("kak_cursor_column")?;
        Ok((self.parse_usize(&line)?, self.parse_usize(&column)?))
    }

    pub fn get_selection(&self) -> Result<String> {
        self.get_variable("kak_selection")
    }

    pub fn get_timestamp(&self) -> Result<usize> {
        let timestamp = self.os_io.get_env_var("kak_timestamp")?;
        self.parse_usize(&timestamp)
    }

    pub fn goto_previous_buffer(&self) {
        self.os_io.print("execute-keys ga\n")
    }

    pub fn parse_cursor(&self, pos: &str) -> Result<(usize, usize)> {
        let (start, end) = pos.split_once('.').context("cursor should contain '.'")?;
        let start = start
            .parse::<usize>()
            .context("could not parse cursor start as an integer")?;
        let end = end
            .parse::<usize>()
            .context("could not parse cursor end as an integer")?;
        Ok((start, end))
    }

    pub fn parse_range_spec(&self, range_spec: &str) -> Result<Vec<(usize, usize, usize)>> {
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

    pub fn get_previous_selection<'a>(
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

    pub fn get_next_selection<'a>(
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
pub(crate) mod tests;
