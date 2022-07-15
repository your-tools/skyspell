use crate::repository::Undoer;
use crate::FakeRepository;

#[test]
fn test_can_undo_global_ignore() {
    let repository = FakeRepository::new();
    let mut undoer = Undoer::new(repository);

    undoer.ignore("foo").unwrap();

    undoer.undo().unwrap();

    assert!(!undoer.is_ignored("foo").unwrap());
}

#[test]
fn test_cannot_undo_twice() {
    let repository = FakeRepository::new();
    let mut undoer = Undoer::new(repository);
    undoer.ignore("foo").unwrap();

    undoer.undo().unwrap();

    undoer.undo().unwrap_err();
}
