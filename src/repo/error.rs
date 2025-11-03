use thiserror::Error;

#[derive(Debug, Error)]
pub enum RepoError {
    #[error("未找到该条目")]
    NotFound,
    #[error("该条目已经存在")]
    AlreadyExists,
    #[error("Database query failed: {0}")]
    TechnicalError(#[from] sqlx::Error),
}
