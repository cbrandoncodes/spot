pub(crate) mod entities;
mod repository;

pub(crate) use entities::draft::Model as DraftRecord;
pub use repository::{FitnessProfileRecord, FitnessProfileRepository, RepositoryError};
