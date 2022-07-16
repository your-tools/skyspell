use crate::IgnoreStore;

pub trait Repository {
    fn repo_method(&mut self);
    fn as_ignore_store(&mut self) -> &mut dyn IgnoreStore;
}
