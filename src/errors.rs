//! # Errors
//!
//! Error types and helper functions used in the application

use std::num::ParseIntError;
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

    #[error("missing the Content-Type header or it has wrong value: {0}")]
    ContentTypeMissingOrWrong(String),

    #[error("couldn't convert Content-Length header's value to a number: {0}")]
    TryFromSliceError(#[from] std::array::TryFromSliceError),

    #[error(transparent)]
    ParseIntError(#[from] ParseIntError),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
