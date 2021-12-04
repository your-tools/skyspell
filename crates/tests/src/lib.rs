pub mod fake_dictionary;
pub mod fake_interactor;
pub mod fake_io;
pub mod fake_repository;

pub use fake_dictionary::FakeDictionary;
pub use fake_interactor::FakeInteractor;
pub use fake_io::FakeIO;
pub use fake_repository::FakeRepository;

#[cfg(test)]
mod tests;
