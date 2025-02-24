//! Custom error type for the xtask crate
//!
//! This enum represents various error types that can occur during the execution of xtask operations.

use libsm::sm4;
use thiserror::Error;

pub type XtaskResult<T> = Result<T, XtaskError>;

/// Custom error type for encryption and firmware generation operations.
#[derive(Error, Debug)]
pub enum XtaskError {
    /// Error for invalid encryption type specification.
    #[error("Invalid encryption type!")]
    InvalidEncryptionType,

    /// Wrapper for standard I/O errors.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Errors from SM4 block cipher operations.
    #[error("SM4 error: {0}")]
    SM4Error(#[from] sm4::error::Sm4Error),

    /// Errors from SM2 public key cryptography operations.
    #[error("SM2 error: {0}")]
    SM2Error(String),

    /// Errors from AES encryption/decryption operations.
    #[error("Aes error: {0}")]
    AesError(String),

    /// Errors from RSA cryptographic operations.
    #[error("RSA error: {0}")]
    RsaError(#[from] rsa::errors::Error),

    /// Errors when parsing RSA key components.
    #[error("RSA parse error: {0}")]
    RsaParseError(String),
}
