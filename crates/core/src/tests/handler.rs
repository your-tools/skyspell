use crate::repository::RepositoryHandler;
use crate::FakeRepository;

#[test]
fn test_can_undo_global_ignore() {
    let repository = FakeRepository::new();
    let mut handler = RepositoryHandler::new(repository);

    handler.ignore("foo").unwrap();

    handler.undo().unwrap();

    assert!(!handler.is_ignored("foo").unwrap());
}

#[test]
fn test_cannot_undo_twice() {
    let repository = FakeRepository::new();
    let mut handler = RepositoryHandler::new(repository);
    handler.ignore("foo").unwrap();

    handler.undo().unwrap();

    handler.undo().unwrap_err();
}
