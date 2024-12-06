//! # Errors
//!
//! Error types and helper functions used in the application

use thiserror::Error;

/// Errors related to working with [`crate::conn`]
#[derive(Debug, Error)]
pub enum ConnectionError {
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("{0}")]
    ParseError(String),

    #[error("{0}")]
    HttpParseError(#[from] httparse::Error),

    #[error("missing the User-Agent header")]
    UserAgentMissing,

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
