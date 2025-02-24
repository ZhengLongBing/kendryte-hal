//! Custom error type for the xtask crate
//!
//! This enum represents various error types that can occur during the execution of xtask operations.
use thiserror::Error;

#[derive(Error, Debug)]
pub enum XtaskError {
    /// Error for invalid encryption type.
    #[error("Invalid encryption type!")]
    InvalidEncryptionType,

    /// Wrapper for standard I/O errors.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Wrapper for clap command-line argument parsing errors.
    #[error("Clap error: {0}")]
    Clap(#[from] clap::Error),
}
