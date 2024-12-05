//! # Errors
//!
//! Error types and helper functions used in the application

use thiserror::Error;

/// Errors related to working with ...
#[derive(Debug, Error)]
pub enum GeneralError {
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
