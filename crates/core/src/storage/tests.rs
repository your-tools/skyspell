use crate::tests::FakeRepository;
use crate::StorageBackend;

#[test]
fn test_can_undo_global_ignore() {
    let repository = FakeRepository::new();
    let mut storage = StorageBackend::Repository(Box::new(repository));
    storage.ignore("foo").unwrap();

    storage.undo().unwrap();

    assert!(!storage.is_ignored("foo").unwrap());
}

#[test]
fn test_cannot_undo_twice() {
    let repository = FakeRepository::new();
    let mut storage = StorageBackend::Repository(Box::new(repository));
    storage.ignore("foo").unwrap();
    storage.undo().unwrap();

    storage.undo().unwrap_err();
}
