use crate::repository::RepositoryHandler;
use crate::FakeRepository;

#[test]
fn test_can_undo_file_name_skip() {
    let repository = FakeRepository::new();
    let mut handler = RepositoryHandler::new(repository);
    handler.skip_file_name("foo.lock").unwrap();

    handler.undo().unwrap();

    assert!(!handler.is_skipped_file_name("foo.lock").unwrap());
}

#[test]
fn test_cannot_undo_twice() {
    let repository = FakeRepository::new();
    let mut handler = RepositoryHandler::new(repository);
    handler.skip_file_name("foo.lock").unwrap();

    handler.undo().unwrap();

    handler.undo().unwrap_err();
}
