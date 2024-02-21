use std::sync::Arc;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("mongo error: {0}")]
    InternalError(#[from] mongodb::error::Error),

    #[error("not found: {details}")]
    NotFound { details: &'static str },

    #[error("invalid update path: {path}")]
    InvalidUpdatePath { path: String },
}

impl From<DatabaseError> for tonic::Status {
    fn from(err: DatabaseError) -> Self {
        match err {
            DatabaseError::NotFound { details } => tonic::Status::not_found(details),
            DatabaseError::InternalError(e) => {
                tracing::error!("mongo error: {}", e);
                let mut s = tonic::Status::internal("error interacting with database");
                s.set_source(Arc::new(e));
                s
            }
            DatabaseError::InvalidUpdatePath { path } => tonic::Status::invalid_argument(path),
        }
    }
}

impl DatabaseError {
    pub fn not_found(details: &'static str) -> Self {
        DatabaseError::NotFound { details }
    }

    pub fn invalid_update_path(path: String) -> Self {
        DatabaseError::InvalidUpdatePath { path }
    }
}