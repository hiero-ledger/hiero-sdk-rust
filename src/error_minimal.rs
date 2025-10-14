// SPDX-License-Identifier: Apache-2.0

//! Minimal error types for WASM builds (no networking)

// Remove unused import

/// Error for WASM builds - simplified without networking errors
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Invalid checksum
    #[error("invalid checksum")]
    InvalidChecksum,

    /// Protobuf conversion error
    #[error("protobuf error: {0}")]
    Protobuf(String),

    /// Basic validation error
    #[error("validation error: {0}")]
    Validation(String),

    /// Transaction pre-check status error (simplified)
    #[error("transaction pre-check failed")]
    TransactionPreCheckStatus { status: i32, cost: Option<u64>, transaction_id: Option<Box<crate::TransactionId>> },

    /// Signature verification error (simplified)
    #[error("signature verification failed: {0}")]
    SignatureVerify(String),

    /// Mnemonic entropy error (simplified)
    #[cfg(feature = "mnemonic")]
    #[error("mnemonic entropy error")]
    MnemonicEntropy,

    /// Mnemonic parse error (simplified)
    #[cfg(feature = "mnemonic")]
    #[error("mnemonic parse error")]
    MnemonicParse { mnemonic: String, reason: String },
    
    /// Basic parsing error
    #[error("parse error: {0}")]
    BasicParse(String),
    
    /// Key parsing error
    #[error("key parse error: {0}")]
    KeyParse(String),
    
    /// Key derivation error
    #[error("key derivation error: {0}")]
    KeyDerive(String),
    
    /// Receipt status error (simplified for WASM)
    #[error("receipt status error")]
    ReceiptStatus { status: i32, transaction_id: crate::TransactionId },
    
    /// Transaction ID not set
    #[error("transaction ID not set")]
    TransactionIdNotSet,
    
    /// Freeze with unset node account IDs
    #[error("freeze transaction requires node account IDs to be set")]
    FreezeUnsetNodeAccountIds,
    
    /// Bad entity ID
    #[error("bad entity ID")]
    BadEntityId,
}

impl Error {
    /// Create protobuf error from string
    pub fn from_protobuf(msg: impl Into<String>) -> Self {
        Self::Protobuf(msg.into())
    }

    /// Create signature verification error
    pub fn signature_verify(msg: impl Into<String>) -> Self {
        Self::SignatureVerify(msg.into())
    }
    
    /// Create basic parse error
    pub fn basic_parse(msg: impl Into<String>) -> Self {
        Self::BasicParse(msg.into())
    }
    
    /// Create key parse error
    pub fn key_parse(msg: impl Into<String>) -> Self {
        Self::KeyParse(msg.into())
    }
    
    /// Create key derivation error
    pub fn key_derive(msg: impl Into<String>) -> Self {
        Self::KeyDerive(msg.into())
    }
}

/// Result type for WASM builds
pub type Result<T> = std::result::Result<T, Error>;

/// Simplified mnemonic entropy error for WASM
#[cfg(feature = "mnemonic")]
pub type MnemonicEntropyError = Error;

/// Simplified mnemonic parse error for WASM
#[cfg(feature = "mnemonic")]
pub type MnemonicParseError = Error;

// Implement From traits for common error conversions needed in WASM
impl From<prost::DecodeError> for Error {
    fn from(err: prost::DecodeError) -> Self {
        Self::Protobuf(err.to_string())
    }
}

impl From<ed25519_dalek::ed25519::Error> for Error {
    fn from(err: ed25519_dalek::ed25519::Error) -> Self {
        Self::SignatureVerify(err.to_string())
    }
}

impl From<hex::FromHexError> for Error {
    fn from(err: hex::FromHexError) -> Self {
        Self::BasicParse(err.to_string())
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(err: std::num::ParseIntError) -> Self {
        Self::BasicParse(err.to_string())
    }
}

impl From<std::num::TryFromIntError> for Error {
    fn from(err: std::num::TryFromIntError) -> Self {
        Self::BasicParse(err.to_string())
    }
}

impl From<time::error::ComponentRange> for Error {
    fn from(err: time::error::ComponentRange) -> Self {
        Self::BasicParse(err.to_string())
    }
}

// Note: Cannot implement From<ErrorType> for String due to orphan rules
// Code that needs to convert errors to String should use .to_string() explicitly 