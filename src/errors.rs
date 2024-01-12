use std::sync::Arc;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("mongo error: {0}")]
    InternalError(#[from] mongodb::error::Error),

    #[error("not found: {details}")]
    NotFound { details: &'static str },
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
        }
    }
}

impl DatabaseError {
    pub fn not_found(details: &'static str) -> Self {
        DatabaseError::NotFound { details }
    }
}
