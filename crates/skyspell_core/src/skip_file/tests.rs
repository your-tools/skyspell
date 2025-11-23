use std::path::PathBuf;

use super::*;

#[test]
fn test_skipping_file_in_subdir() {
    let this_dir = PathBuf::from(".");
    let gitignore = GitignoreBuilder::new(this_dir)
        .add_line(None, "foo/")
        .unwrap()
        .build()
        .unwrap();
    let actual = gitignore.matched_path_or_any_parents("foo/bar", false);
    assert!(actual.is_ignore());
}
