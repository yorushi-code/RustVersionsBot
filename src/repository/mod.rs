pub mod version_repo;
pub mod client_repo;
pub mod admin_repo;
pub mod settings_repo;

pub use version_repo::VersionRepo;
pub use client_repo::ClientRepo;
pub use admin_repo::AdminRepo;
pub use settings_repo::SettingsRepo;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum RepoError {
    #[error("not found")]
    NotFound,
    #[error("already exists")]
    AlreadyExists,
    #[error("database error: {0}")]
    Db(#[from] sea_orm::DbErr),
}

pub type RepoResult<T> = Result<T, RepoError>;