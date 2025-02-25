//! Firmware generation module for K230 platform.
//!
//! This module provides functionality to generate encrypted and signed firmware
//! packages for the K230 platform. It supports multiple encryption types including
//! SM4 and AES, along with RSA and SM2 signatures.

use crate::error::{XtaskError, XtaskResult};
use crate::gen::config::{
    ADD_AUTH_DATA, D, E, ID, ID_LEN, INITIAL_AES_IV, INITIAL_AES_KEY, MAGIC, N, PRIVATE_KEY,
    PUBLIC_KEY, PUBLIC_KEY_X, PUBLIC_KEY_Y, SM4_IV, SM4_KEY, VERSION,
};
use aes_gcm::aead::OsRng;
use aes_gcm::{AeadInPlace, Aes256Gcm, Key, KeyInit, Nonce};
use libsm::sm2::ecc::EccCtx;
use libsm::sm2::signature::SigCtx;
use libsm::sm3::hash::Sm3Hash;
use libsm::sm4::cipher_mode::CipherMode;
use libsm::sm4::Cipher;
use num_bigint_dig::BigUint;
use rsa::traits::SignatureScheme;
use rsa::{Pkcs1v15Sign, RsaPrivateKey};
use sha2::{Digest, Sha256};
use std::str::FromStr;

/// Encryption types supported for firmware.
#[derive(Debug, Default, Clone, Copy)]
pub enum EncryptionType {
    #[default]
    None = 0,
    Sm4 = 1,
    Aes = 2,
}

impl FromStr for EncryptionType {
    type Err = XtaskError;

    /// Parse encryption type from string.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "none" => Ok(Self::None),
            "sm4" => Ok(Self::Sm4),
            "aes" => Ok(Self::Aes),
            _ => Err(XtaskError::InvalidEncryptionType),
        }
    }
}

/// Generate firmware with specified data and encryption type.
///
/// This function takes the input data and an encryption type, and generates
/// a firmware package with the appropriate encryption and signature.
pub fn gen_firmware(data: &[u8], encryption: EncryptionType) -> XtaskResult<Vec<u8>> {
    // Prepend version information to the input data
    let mut data_with_version = vec![];
    data_with_version.extend_from_slice(VERSION);
    data_with_version.extend_from_slice(data);

    // Initialize firmware package with magic bytes
    let mut firmware = vec![];
    firmware.extend_from_slice(MAGIC.as_bytes());
    println!("the magic is: {}", MAGIC);

    match encryption {
        EncryptionType::None => {
            println!("----- NO ENCRYPTION + HASH-256 -----");
            // Calculate and store data length (4 bytes, little-endian)
            let data_len = data_with_version.len() as i32;
            let data_len_bytes: [u8; 4] = data_len.to_ne_bytes();
            firmware.extend_from_slice(&data_len_bytes);

            // Store encryption type (4 bytes, little-endian)
            let encryption_bytes: [u8; 4] = (encryption as i32).to_le_bytes();
            firmware.extend_from_slice(&encryption_bytes);
            println!("the encryption type: {}", encryption as i32);

            // Calculate SHA-256 hash of data and add to firmware
            let data_with_version_hash = sha_256(&data_with_version);
            firmware.extend_from_slice(&data_with_version_hash);

            // Add padding to align with firmware format (516 - 32 bytes)
            let padding = vec![0; 516 - 32];
            firmware.extend_from_slice(&padding);

            // Append the actual data
            firmware.extend_from_slice(&data_with_version);
        }
        EncryptionType::Sm4 => {
            println!("----- SM4-CBC + SM2 -----");
            // Encrypt data using SM4 in CBC mode
            let cipher = Cipher::new(SM4_KEY, CipherMode::Cbc)?;
            let ciphertext = cipher.encrypt(ADD_AUTH_DATA, &data_with_version, SM4_IV)?;

            // Store encrypted data length and encryption type
            let data_len = ciphertext.len() as i32;
            let data_len_bytes = data_len.to_le_bytes();
            firmware.extend_from_slice(&data_len_bytes);
            let encryption_bytes: [u8; 4] = (encryption as i32).to_le_bytes();
            firmware.extend_from_slice(&encryption_bytes);
            println!("the encryption type: {}", encryption as i32);

            // Initialize SM2 signature context and load keys
            let sig_ctx = SigCtx::new();
            let pk = sig_ctx
                .load_pubkey(PUBLIC_KEY)
                .map_err(|e| XtaskError::SM2Error(e.to_string()))?;
            let sk = sig_ctx
                .load_seckey(PRIVATE_KEY)
                .map_err(|e| XtaskError::SM2Error(e.to_string()))?;

            // Initialize elliptic curve context for SM2
            let ecc_ctx = EccCtx::new();

            // Get curve parameters for SM3 hash calculation
            let a = ecc_ctx.get_a().to_bytes();
            let b = ecc_ctx.get_b().to_bytes();
            let g = ecc_ctx
                .generator()
                .map_err(|e| XtaskError::SM2Error(e.to_string()))?;
            let x_g = g.x.to_bytes();
            let y_g = g.y.to_bytes();

            // Prepare Z value for SM2 signature (user ID and curve parameters)
            let mut z = vec![];
            z.extend_from_slice(ID_LEN);
            z.extend_from_slice(ID.as_bytes());
            z.extend_from_slice(&a);
            z.extend_from_slice(&b);
            z.extend_from_slice(&x_g);
            z.extend_from_slice(&y_g);
            z.extend_from_slice(PUBLIC_KEY);
            let z_a = Sm3Hash::new(&z).get_hash();

            // Calculate message hash for signing
            let mut m = vec![];
            m.extend_from_slice(&z_a);
            m.extend_from_slice(&ciphertext);
            let e = Sm3Hash::new(&m).get_hash();

            // TODO: Use a fixed K value for signing
            // Generate SM2 signature
            let digest = sig_ctx
                .hash(ID, &pk, &e)
                .map_err(|e| XtaskError::SM2Error(e.to_string()))?;
            let sign = sig_ctx
                .sign_raw(&digest[..], &sk)
                .map_err(|e| XtaskError::SM2Error(e.to_string()))?;

            // Extract signature components (r,s)
            let r = sign.get_r().to_bytes_le();
            let s = sign.get_s().to_bytes_le();

            // Combine signature components
            let mut sign = vec![];
            sign.extend_from_slice(&r);
            sign.extend_from_slice(&s);

            // Display signature components for debugging
            display_bytes("sign:", &sign);
            display_bytes("r:", &r);
            display_bytes("s:", &s);

            // Add ID information to firmware
            let id = ID.as_bytes();
            let id_len = id.len() as i32;
            let id_len_bytes = id_len.to_le_bytes();
            firmware.extend_from_slice(&id_len_bytes);
            firmware.extend_from_slice(id);

            // Add padding to align with firmware format
            let padding = vec![0; 512 - 32 * 4 - id.len()];
            firmware.extend_from_slice(&padding);

            // Add public key components and signature
            firmware.extend_from_slice(PUBLIC_KEY_X);
            firmware.extend_from_slice(PUBLIC_KEY_Y);
            firmware.extend_from_slice(&r);
            firmware.extend_from_slice(&s);

            // Calculate and display SM2 public key hash for verification
            let mut sm2_pub_key = vec![];
            sm2_pub_key.extend_from_slice(&id_len_bytes);
            sm2_pub_key.extend_from_slice(&id);
            sm2_pub_key.extend_from_slice(PUBLIC_KEY_X);
            sm2_pub_key.extend_from_slice(PUBLIC_KEY_Y);
            let sm2_pub_key_hash = Sm3Hash::new(&sm2_pub_key).get_hash();
            display_bytes("the hash value of sm2 puk-key is: ", &sm2_pub_key_hash);

            // Add encrypted data
            firmware.extend_from_slice(&ciphertext);
        }
        EncryptionType::Aes => {
            println!("----- AES-GCM + RSA-2048 -----");
            // Initialize AES-GCM encryption with key and nonce
            let key = Key::<Aes256Gcm>::from_slice(INITIAL_AES_KEY);
            let nonce = Nonce::from_slice(INITIAL_AES_IV);
            let cipher = Aes256Gcm::new(key);
            let mut ciphertext = data_with_version.to_vec();

            // Perform AES-GCM encryption and get authentication tag
            let tag = cipher
                .encrypt_in_place_detached(nonce, ADD_AUTH_DATA, &mut ciphertext)
                .map_err(|e| XtaskError::AesError(e.to_string()))?;
            ciphertext.extend_from_slice(&tag);

            // Store encrypted data length and encryption type
            let data_len = ciphertext.len() as i32;
            let data_len_bytes = data_len.to_le_bytes();
            firmware.extend_from_slice(&data_len_bytes);
            let encryption_bytes: [u8; 4] = (encryption as i32).to_le_bytes();
            firmware.extend_from_slice(&encryption_bytes);
            println!("the encryption type: {}", encryption as i32);

            // Parse RSA key components
            let n = BigUint::parse_bytes(N, 16).ok_or(XtaskError::RsaParseError(
                "Failed to parse N for RSA".to_string(),
            ))?;
            let e = u32::from_str_radix(&E[2..], 16)
                .map_err(|_| XtaskError::RsaParseError("Failed to parse E for RSA".to_string()))?;
            let e = BigUint::from(e);
            let d = BigUint::parse_bytes(D, 16).ok_or(XtaskError::RsaParseError(
                "Failed to parse D for RSA".to_string(),
            ))?;

            // Create RSA private key from components
            let private_key = RsaPrivateKey::from_components(
                n.clone(),
                e.clone(),
                d.clone(),
                Vec::new(), // Prime factors omitted for simplicity
            )?;

            display_bytes("tag:", &tag);

            // Generate RSA signature using PKCS#1 v1.5 padding
            let tag_hash = sha_256(&tag);
            let pkcs1_15 = Pkcs1v15Sign::new_unprefixed();
            let signature = pkcs1_15.sign::<OsRng>(None, &private_key, &tag_hash)?;

            // Add RSA public key components to firmware
            let n_bytes = n.to_bytes_le();
            let e_bytes = e.to_bytes_le();
            firmware.extend_from_slice(&n_bytes);
            firmware.extend_from_slice(&e_bytes);

            // Add RSA signature
            firmware.extend_from_slice(&signature);

            // Calculate and display RSA public key hash for verification
            let mut pub_key = vec![];
            pub_key.extend_from_slice(&n_bytes);
            pub_key.extend_from_slice(&e_bytes);
            let pub_key_hash = sha_256(&pub_key);
            display_bytes("the hash value of RSA puk-key is: ", &pub_key_hash);

            // Add encrypted data
            firmware.extend_from_slice(&ciphertext);
        }
    }

    Ok(firmware)
}

/// Display bytes as hexadecimal string.
fn display_bytes(prefix: &str, bytes: &[u8]) {
    println!("{}", prefix);
    let bytes_hex_str = hex::encode(bytes);
    println!("{}", bytes_hex_str);
}

/// Calculate SHA-256 hash of input data.
fn sha_256(data: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}
