use anyhow::{anyhow, Context, Result};

use std::path::PathBuf;

use crate::kak::checker::{SKYSPELL_LANG_OPT, SKYSPELL_PROJECT_OPT};
use crate::Project;

#[allow(dead_code)]
pub(crate) fn debug(message: &str) {
    println!("echo -debug {}", message);
}

pub(crate) fn get_from_environ(key: &str) -> Result<String> {
    std::env::var(key).map_err(|_| anyhow!("{} not found in environment", key))
}

pub(crate) fn get_option(name: &str) -> Result<String> {
    std::env::var(format!("kak_opt_{}", name)).map_err(|_| anyhow!("{} option not defined", name))
}

pub(crate) fn parse_usize(v: &str) -> Result<usize> {
    v.parse()
        .map_err(|_| anyhow!("could not parse {} as a positive number"))
}

pub(crate) fn get_cursor() -> Result<(usize, usize)> {
    let line = get_from_environ("kak_cursor_line")?;
    let column = get_from_environ("kak_cursor_column")?;
    Ok((parse_usize(&line)?, parse_usize(&column)?))
}

pub(crate) fn get_selection() -> Result<String> {
    get_from_environ("kak_selection")
}

pub(crate) fn get_lang() -> Result<String> {
    get_option(SKYSPELL_LANG_OPT)
}

pub(crate) fn get_project() -> Result<Project> {
    let as_str = get_option(SKYSPELL_PROJECT_OPT)?;
    let path = PathBuf::from(as_str);
    Project::new(&path)
}

pub(crate) fn goto_previous_buffer() {
    println!("execute-keys ga")
}

pub(crate) fn parse_cursor(pos: &str) -> Result<(usize, usize)> {
    let (start, end) = pos.split_once('.').context("cursor should contain '.'")?;
    let start = start
        .parse::<usize>()
        .context("could not parse cursor start as an integer")?;
    let end = end
        .parse::<usize>()
        .context("could not parse cursor end as an integer")?;
    Ok((start, end))
}

pub(crate) fn parse_range_spec(range_spec: &str) -> Result<Vec<(usize, usize, usize)>> {
    // range-spec is empty
    if range_spec == "0" {
        return Ok(vec![]);
    }

    // Skip the timestamp
    let mut split = range_spec.split_whitespace();
    split.next();

    split.into_iter().map(|x| parse_range(x)).collect()
}

fn parse_range(range: &str) -> Result<(usize, usize, usize)> {
    let (range, _face) = range
        .split_once('|')
        .context("range spec should contain a face")?;
    let (start, end) = range
        .split_once(',')
        .context("range spec should contain ','")?;

    let (start_line, start_col) = parse_cursor(start)?;
    let (_end_line, end_col) = parse_cursor(end)?;

    Ok((start_line, start_col, end_col))
}

pub(crate) fn get_previous_selection(
    cursor: (usize, usize),
    ranges: &[(usize, usize, usize)],
) -> Option<&(usize, usize, usize)> {
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

pub(crate) fn get_next_selection(
    cursor: (usize, usize),
    ranges: &[(usize, usize, usize)],
) -> Option<&(usize, usize, usize)> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn goto_next_() {
        let pos = (1, 21);
        let ranges = [(1, 12, 19), (2, 19, 27)];
        let actual = get_previous_selection(pos, &ranges).unwrap();
        assert_eq!(actual, &(1, 12, 19));
    }
}
