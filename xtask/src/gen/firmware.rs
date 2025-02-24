use crate::error::XtaskError;
use crate::error::XtaskError::InvalidEncryptionType;
use sha2::{Digest, Sha256};
use std::fmt::Display;
use std::str::FromStr;

// Magic bytes for K230 firmware
const MAGIC: &[u8] = b"K230";
// Version of the firmware format
const VERSION: [u8; 4] = [0, 0, 0, 0];

/// Encryption types supported for firmware.
#[derive(Debug, Default, Clone, Copy)]
pub enum Encryption {
    #[default]
    None = 0,
    Sm4 = 1,
    AesRsa = 2,
}

impl FromStr for Encryption {
    type Err = XtaskError;

    /// Parse encryption type from string.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "none" => Ok(Self::None),
            "sm4" => Ok(Self::Sm4),
            "aesrsa" => Ok(Self::AesRsa),
            _ => Err(InvalidEncryptionType),
        }
    }
}

impl Display for Encryption {
    /// Convert encryption type to string.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "none"),
            Self::Sm4 => write!(f, "sm4"),
            Self::AesRsa => write!(f, "aesrsa"),
        }
    }
}

/// Encrypt data using specified encryption type.
fn encrypt(data: &[u8], encryption_type: Encryption) -> Result<Vec<u8>, XtaskError> {
    match encryption_type {
        Encryption::None => Ok(data.to_vec()), // No encryption
        Encryption::Sm4 => todo!(),            // TODO: Implement SM4 encryption
        Encryption::AesRsa => todo!(),         // TODO: Implement AES-RSA encryption
    }
}

/// Calculate SHA256 hash of the message.
fn sha256(message: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(message);
    hasher.finalize().to_vec()
}

/// Generate firmware with specified data and encryption type.
pub fn gen_firmware(data: &[u8], encryption_type: Encryption) -> Result<Vec<u8>, XtaskError> {
    // Prepare input data with version
    let mut input_data = VERSION.to_vec();
    input_data.extend_from_slice(&data);

    // Calculate lengths and convert to bytes
    let data_len = input_data.len() as u32;
    let raw_data_len: [u8; 4] = data_len.to_le_bytes();
    let encryption_type_bytes: [u8; 4] = (encryption_type as u32).to_le_bytes();

    // Encrypt data and calculate hash
    let encrypted_data = encrypt(&mut input_data, encryption_type)?;
    let hash_data = sha256(&encrypted_data);

    // Construct firmware
    let mut firmware = Vec::new();
    firmware.extend_from_slice(MAGIC);
    firmware.extend_from_slice(&raw_data_len);
    firmware.extend_from_slice(&encryption_type_bytes);
    firmware.extend_from_slice(&hash_data);

    firmware.extend(vec![0; 516 - 32]); // Add padding
    firmware.extend_from_slice(&input_data);

    Ok(firmware)
}
