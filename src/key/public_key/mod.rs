// SPDX-License-Identifier: Apache-2.0

use std::fmt::{
    self,
    Debug,
    Display,
    Formatter,
};
use std::hash::{
    Hash,
    Hasher,
};
use std::str::FromStr;

use ed25519_dalek::Verifier as _;
use hedera_proto::services;
use hmac::digest::generic_array::sequence::Split;
use hmac::digest::generic_array::GenericArray;
use k256::ecdsa;
use k256::ecdsa::signature::DigestVerifier as _;
use pkcs8::der::asn1::BitStringRef;
use pkcs8::der::{
    Decode,
    Encode,
};
use pkcs8::ObjectIdentifier;
use prost::Message;
use sha2::Digest;

use crate::key::private_key::{
    ED25519_OID,
    K256_OID,
};
use crate::protobuf::ToProtobuf;
use crate::signer::AnySigner;
use crate::transaction::TransactionSources;
use crate::{
    AccountId,
    Error,
    EvmAddress,
    FromProtobuf,
    Transaction,
};

#[cfg(test)]
mod tests;

pub(super) const EC_ALGORITM_OID: ObjectIdentifier =
    ObjectIdentifier::new_unwrap("1.2.840.10045.2.1");

/// A public key on the Hiero network.
#[derive(Clone, Eq, Copy, Hash, PartialEq)]
pub struct PublicKey(PublicKeyData);

#[derive(Clone, Copy)]
enum PublicKeyData {
    Ed25519(ed25519_dalek::VerifyingKey),
    Ecdsa(k256::ecdsa::VerifyingKey),
}

impl Hash for PublicKeyData {
    fn hash<H: Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
        match &self {
            PublicKeyData::Ed25519(key) => key.to_bytes().hash(state),
            PublicKeyData::Ecdsa(key) => key.to_encoded_point(true).as_bytes().hash(state),
        }
    }
}

impl PartialEq for PublicKeyData {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Ed25519(l0), Self::Ed25519(r0)) => l0 == r0,
            (Self::Ecdsa(l0), Self::Ecdsa(r0)) => l0 == r0,
            _ => false,
        }
    }
}

impl Eq for PublicKeyData {}

impl PublicKey {
    pub(super) fn ed25519(key: ed25519_dalek::VerifyingKey) -> Self {
        Self(PublicKeyData::Ed25519(key))
    }

    pub(super) fn ecdsa(key: k256::ecdsa::VerifyingKey) -> Self {
        Self(PublicKeyData::Ecdsa(key))
    }

    /// Returns `true` if the public key is `Ed25519`.
    #[must_use]
    pub fn is_ed25519(&self) -> bool {
        matches!(&self.0, PublicKeyData::Ed25519(_))
    }

    /// Returns `true` if the public key data is `Ecdsa`.
    #[must_use]
    pub fn is_ecdsa(&self) -> bool {
        matches!(&self.0, PublicKeyData::Ecdsa(_))
    }

    pub(crate) fn from_alias_bytes(bytes: &[u8]) -> crate::Result<Option<Self>> {
        if bytes.is_empty() {
            return Ok(None);
        }
        Ok(Some(PublicKey::from_protobuf(
            services::Key::decode(bytes).map_err(Error::from_protobuf)?,
        )?))
    }

    /// Parse a `PublicKey` from a sequence of bytes.
    ///
    /// # Errors
    /// - [`Error::KeyParse`] if `bytes` cannot be parsed into a `PublicKey`.
    pub fn from_bytes(bytes: &[u8]) -> crate::Result<Self> {
        if bytes.len() == 32 {
            return Self::from_bytes_ed25519(bytes);
        }

        if bytes.len() == 33 {
            return Self::from_bytes_ecdsa(bytes);
        }

        Self::from_bytes_der(bytes)
    }

    /// Parse a Ed25519 `PublicKey` from a sequence of bytes.   
    ///
    /// # Errors
    /// - [`Error::KeyParse`] if `bytes` cannot be parsed into a ed25519 `PublicKey`.
    pub fn from_bytes_ed25519(bytes: &[u8]) -> crate::Result<Self> {
        let data = if let Ok(bytes) = bytes.try_into() {
            ed25519_dalek::VerifyingKey::from_bytes(bytes).map_err(Error::key_parse)?
        } else {
            return Self::from_bytes_der(bytes);
        };

        Ok(Self::ed25519(data))
    }

    /// Parse a ECDSA(secp256k1) `PublicKey` from a sequence of bytes.
    ///
    /// # Errors
    /// - [`Error::KeyParse`] if `bytes` cannot be parsed into a ECDSA(secp256k1) `PublicKey`.
    pub fn from_bytes_ecdsa(bytes: &[u8]) -> crate::Result<Self> {
        let data = if bytes.len() == 32 + 1 || bytes.len() == 32 * 2 + 1 {
            k256::ecdsa::VerifyingKey::from_sec1_bytes(bytes).map_err(Error::key_parse)?
        } else {
            return Self::from_bytes_der(bytes);
        };

        Ok(Self::ecdsa(data))
    }

    /// Parse a `PublicKey` from a sequence of der encoded bytes.
    ///
    /// # Errors
    /// - [`Error::KeyParse`] if `bytes` cannot be parsed into a `PublicKey`.
    pub fn from_bytes_der(bytes: &[u8]) -> crate::Result<Self> {
        let info = pkcs8::SubjectPublicKeyInfoRef::from_der(bytes)
            .map_err(|err| Error::key_parse(err.to_string()))?;

        let bytes = info
            .subject_public_key
            .as_bytes()
            .ok_or_else(|| Error::key_parse("Unexpected bitstring len"))?;

        match info.algorithm.oid {
            K256_OID => Self::from_bytes_ecdsa(bytes),
            EC_ALGORITM_OID if info.algorithm.parameters_oid().ok() == Some(K256_OID) => {
                Self::from_bytes_ecdsa(bytes)
            }
            ED25519_OID => Self::from_bytes_ed25519(bytes),
            oid => Err(Error::key_parse(format!("unsupported key algorithm: {oid}"))),
        }
    }

    /// Decodes `self` from a der encoded `str`
    ///
    /// Optionally strips a `0x` prefix.
    /// See [`from_bytes_der`](Self::from_bytes_der)
    ///
    /// # Errors
    /// - [`Error::KeyParse`] if `s` cannot be parsed into a `PublicKey`.
    pub fn from_str_der(s: &str) -> crate::Result<Self> {
        Self::from_bytes_der(
            &hex::decode(s.strip_prefix("0x").unwrap_or(s)).map_err(Error::key_parse)?,
        )
    }

    /// Parse a Ed25519 `PublicKey` from a string containing the raw key material.
    ///
    /// Optionally strips a `0x` prefix.
    ///
    /// # Errors
    /// - [`Error::KeyParse`] if `s` cannot be parsed into a ed25519 `PublicKey`.
    pub fn from_str_ed25519(s: &str) -> crate::Result<Self> {
        Self::from_bytes_ed25519(
            &hex::decode(s.strip_prefix("0x").unwrap_or(s)).map_err(Error::key_parse)?,
        )
    }

    /// Parse a ECDSA(secp256k1) `PublicKey` from a string containing the raw key material.
    ///
    /// Optionally strips a `0x` prefix.
    ///
    /// # Errors
    /// - [`Error::KeyParse`] if `s` cannot be parsed into a Ecdsa(secp256k1) `PublicKey`.
    pub fn from_str_ecdsa(s: &str) -> crate::Result<Self> {
        Self::from_bytes_ecdsa(
            &hex::decode(s.strip_prefix("0x").unwrap_or(s)).map_err(Error::key_parse)?,
        )
    }

    /// Return this `PublicKey`, serialized as bytes.
    ///
    /// If this is an ed25519 public key, this is equivalent to [`to_bytes_raw`](Self::to_bytes_raw)
    /// If this is an ecdsa public key, this is equivalent to [`to_bytes_der`](Self::to_bytes_der)
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        match &self.0 {
            PublicKeyData::Ed25519(_) => self.to_bytes_raw(),
            PublicKeyData::Ecdsa(_) => self.to_bytes_der(),
        }
    }

    /// Return this `PublicKey`, serialized as der-encoded bytes.
    // panic should be impossible (`unreachable`)
    #[allow(clippy::missing_panics_doc)]
    #[must_use]
    pub fn to_bytes_der(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(64);

        match &self.0 {
            PublicKeyData::Ed25519(key) => {
                let key = key.to_bytes();
                let info = pkcs8::SubjectPublicKeyInfoRef {
                    algorithm: self.algorithm(),
                    subject_public_key: BitStringRef::from_bytes(&key).unwrap(),
                };

                info.encode_to_vec(&mut buf).unwrap();
            }

            PublicKeyData::Ecdsa(key) => {
                let key = key.to_encoded_point(true);
                let info = pkcs8::SubjectPublicKeyInfoRef {
                    algorithm: self.algorithm(),
                    subject_public_key: BitStringRef::from_bytes(key.as_bytes()).unwrap(),
                };

                info.encode_to_vec(&mut buf).unwrap();
            }
        }

        buf
    }

    fn algorithm(&self) -> pkcs8::AlgorithmIdentifierRef<'_> {
        pkcs8::AlgorithmIdentifierRef {
            parameters: None,
            oid: match self.0 {
                PublicKeyData::Ed25519(_) => ED25519_OID,
                PublicKeyData::Ecdsa(_) => K256_OID,
            },
        }
    }

    /// Return this `PublicKey`, serialized as bytes.
    #[must_use]
    pub fn to_bytes_raw(&self) -> Vec<u8> {
        match &self.0 {
            PublicKeyData::Ed25519(key) => key.to_bytes().as_slice().to_vec(),
            PublicKeyData::Ecdsa(key) => key.to_encoded_point(true).to_bytes().into_vec(),
        }
    }

    /// DER encodes self, then hex encodes the result.
    #[must_use]
    pub fn to_string_der(&self) -> String {
        hex::encode(self.to_bytes_der())
    }

    /// Returns the raw bytes of `self` after hex encoding.
    #[must_use]
    pub fn to_string_raw(&self) -> String {
        hex::encode(self.to_bytes_raw())
    }

    /// Creates an [`AccountId`] with the given `shard`, `realm`, and `self` as an [`alias`](AccountId.alias).
    ///
    /// # Examples
    ///
    /// ```
    /// use hedera::PublicKey;
    ///
    /// let key: PublicKey = "302d300706052b8104000a03220002703a9370b0443be6ae7c507b0aec81a55e94e4a863b9655360bd65358caa6588".parse().unwrap();
    ///
    /// let account_id = key.to_account_id(0, 0);
    /// assert_eq!(account_id.to_string(), "0.0.302d300706052b8104000a03220002703a9370b0443be6ae7c507b0aec81a55e94e4a863b9655360bd65358caa6588");
    /// ```
    #[must_use]
    pub fn to_account_id(&self, shard: u64, realm: u64) -> AccountId {
        AccountId { shard, realm, alias: Some(*self), evm_address: None, num: 0, checksum: None }
    }

    /// Convert this public key into an evm address.
    /// The EVM address is This is the rightmost 20 bytes of the 32 byte Keccak-256 hash of the ECDSA public key.
    ///
    /// Returns `Some(evm_address)` if `self.is_ecdsa`, otherwise `None`.
    #[must_use]
    pub fn to_evm_address(&self) -> Option<EvmAddress> {
        if let PublicKeyData::Ecdsa(ecdsa_key) = &self.0 {
            // we specifically want the uncompressed form ...
            let encoded_point = ecdsa_key.to_encoded_point(false);
            let bytes = encoded_point.as_bytes();
            // ... and without the tag (04):
            let bytes = &bytes[1..];
            let hash = sha3::Keccak256::digest(bytes);

            let (_, sliced): (GenericArray<u8, hmac::digest::typenum::U12>, _) = hash.split();

            let sliced: [u8; 20] = sliced.into();
            Some(EvmAddress::from(sliced))
        } else {
            None
        }
    }

    /// Verify a `signature` on a `msg` with this public key.
    ///
    /// # Errors
    /// - [`Error::SignatureVerify`] if the signature algorithm doesn't match this `PublicKey`.
    /// - [`Error::SignatureVerify`] if the signature is invalid for this `PublicKey`.
    pub fn verify(&self, msg: &[u8], signature: &[u8]) -> crate::Result<()> {
        match &self.0 {
            PublicKeyData::Ed25519(key) => {
                let signature = ed25519_dalek::Signature::try_from(signature)
                    .map_err(Error::signature_verify)?;

                key.verify(msg, &signature).map_err(Error::signature_verify)
            }
            PublicKeyData::Ecdsa(key) => {
                let signature =
                    ecdsa::Signature::try_from(signature).map_err(Error::signature_verify)?;

                key.verify_digest(sha3::Keccak256::new_with_prefix(msg), &signature)
                    .map_err(Error::signature_verify)
            }
        }
    }

    pub(crate) fn verify_transaction_sources(
        &self,
        sources: &TransactionSources,
    ) -> crate::Result<()> {
        use services::signature_pair::Signature;
        let pk_bytes = self.to_bytes_raw();

        for signed_transaction in sources.signed_transactions() {
            let mut found = false;
            for sig_pair in
                signed_transaction.sig_map.as_ref().map_or_else(|| [].as_slice(), |it| &it.sig_pair)
            {
                if !pk_bytes.starts_with(&sig_pair.pub_key_prefix) {
                    continue;
                }

                found = true;
                let Some(Signature::EcdsaSecp256k1(sig) | Signature::Ed25519(sig)) =
                    &sig_pair.signature
                else {
                    return Err(Error::signature_verify("Unsupported transaction signature type"));
                };

                self.verify(&signed_transaction.body_bytes, sig)?;
            }

            if !found {
                return Err(Error::signature_verify("signer not in transaction"));
            }
        }

        Ok(())
    }

    /// Returns `Ok(())` if this public key has signed the given transaction.
    ///
    /// # Errors
    /// - [`Error::SignatureVerify`] if the private key associated with this public key did _not_ sign this transaction,
    ///   or the signature associated was invalid.
    pub fn verify_transaction<D: crate::transaction::TransactionExecute>(
        &self,
        transaction: &mut Transaction<D>,
    ) -> crate::Result<()> {
        if transaction.signers().map(AnySigner::public_key).any(|it| self == &it) {
            return Ok(());
        }

        transaction.freeze()?;

        let Some(sources) = transaction.sources() else {
            return Err(Error::signature_verify("signer not in transaction"));
        };

        self.verify_transaction_sources(sources)
    }

    #[must_use]
    pub(crate) fn kind(&self) -> super::KeyKind {
        match &self.0 {
            PublicKeyData::Ed25519(_) => super::KeyKind::Ed25519,
            PublicKeyData::Ecdsa(_) => super::KeyKind::Ecdsa,
        }
    }
}

impl Debug for PublicKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "\"{self}\"")
    }
}

impl Display for PublicKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.pad(&self.to_string_der())
    }
}

impl FromStr for PublicKey {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_bytes(&hex::decode(s.strip_prefix("0x").unwrap_or(s)).map_err(Error::key_parse)?)
    }
}

impl FromProtobuf<services::Key> for PublicKey {
    fn from_protobuf(pb: services::Key) -> crate::Result<Self>
    where
        Self: Sized,
    {
        use services::key::Key::*;

        match pb.key {
            Some(Ed25519(bytes)) => PublicKey::from_bytes_ed25519(&bytes),
            Some(ContractId(_)) => {
                Err(Error::from_protobuf("unexpected unsupported Contract ID key in single key"))
            }
            Some(DelegatableContractId(_)) => Err(Error::from_protobuf(
                "unexpected unsupported Delegatable Contract ID key in single key",
            )),
            Some(Rsa3072(_)) => {
                Err(Error::from_protobuf("unexpected unsupported RSA-3072 key in single key"))
            }
            Some(Ecdsa384(_)) => {
                Err(Error::from_protobuf("unexpected unsupported ECDSA-384 key in single key"))
            }
            Some(ThresholdKey(_)) => {
                Err(Error::from_protobuf("unexpected threshold key as single key"))
            }
            Some(KeyList(_)) => Err(Error::from_protobuf("unexpected key list as single key")),
            Some(EcdsaSecp256k1(bytes)) => PublicKey::from_bytes_ecdsa(&bytes),
            None => Err(Error::from_protobuf("unexpected empty key in single key")),
        }
    }
}

impl ToProtobuf for PublicKey {
    type Protobuf = services::Key;

    fn to_protobuf(&self) -> Self::Protobuf {
        let key = match self.kind() {
            super::KeyKind::Ed25519 => services::key::Key::Ed25519(self.to_bytes_raw()),
            super::KeyKind::Ecdsa => services::key::Key::EcdsaSecp256k1(self.to_bytes_raw()),
        };

        Self::Protobuf { key: Some(key) }
    }
}

// TODO: to_protobuf
// TODO: verify_transaction
