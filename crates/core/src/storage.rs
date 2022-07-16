/// We have two backends to store ignore words
/// One can manipulate ignored words, but the
/// other is more powerful because it can store
/// and retriev operations

/// Thus, we crate an enum to represent the
/// "capabilities" of a storage - either it implements Repository with
/// all its methods, or it implements IgnoreStore with a subset of
/// these.
use crate::{IgnoreStore, Repository};

pub enum Storage {
    IgnoreStore(Box<dyn IgnoreStore>),
    Repository(Box<dyn Repository>),
}

impl Storage {
    pub(crate) fn as_ignore_store(&mut self) -> &mut dyn IgnoreStore {
        match self {
            Storage::IgnoreStore(i) => i.as_mut(),
            Storage::Repository(r) => r.as_ignore_store(),
        }
    }

    pub(crate) fn as_repository(&mut self) -> Option<&mut dyn Repository> {
        match self {
            Storage::IgnoreStore(_) => None,
            Storage::Repository(r) => Some(r.as_mut()),
        }
    }
}
