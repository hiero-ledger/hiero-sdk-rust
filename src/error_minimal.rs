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
    MnemonicParse,
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
}

/// Result type for WASM builds
pub type Result<T> = std::result::Result<T, Error>;

/// Simplified mnemonic entropy error for WASM
#[cfg(feature = "mnemonic")]
pub type MnemonicEntropyError = Error;

/// Simplified mnemonic parse error for WASM
#[cfg(feature = "mnemonic")]
pub type MnemonicParseError = Error; 