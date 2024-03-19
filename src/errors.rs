use std::sync::Arc;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApplicationError {
    #[error("mongodb error: {0}")]
    MongoError(#[from] mongodb::error::Error),

    #[error("rabbitmq error: {0}")]
    RabbitError(#[from] amqprs::error::Error),

    #[error("not found: {details}")]
    NotFound { details: &'static str },

    #[error("invalid update path: {path}")]
    InvalidUpdatePath { path: String },
}

impl From<ApplicationError> for tonic::Status {
    fn from(error: ApplicationError) -> Self {
        match error {
            ApplicationError::NotFound { details } => tonic::Status::not_found(details),
            ApplicationError::InvalidUpdatePath { path } => tonic::Status::invalid_argument(path),
            _ => {
                tracing::error!(%error, "internal error");
                let mut s = tonic::Status::internal("internal error");
                s.set_source(Arc::new(error));
                s
            }
        }
    }
}

impl ApplicationError {
    pub fn not_found(details: &'static str) -> Self {
        ApplicationError::NotFound { details }
    }

    pub fn invalid_update_path(path: String) -> Self {
        ApplicationError::InvalidUpdatePath { path }
    }
}
