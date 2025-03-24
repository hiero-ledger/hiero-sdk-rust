// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
mod tests;

use std::fmt::{
    Debug,
    Display,
    Formatter,
};
use std::str::FromStr;
use std::{
    fmt,
    mem,
};

use aes::cipher::generic_array::GenericArray;
use aes::cipher::{
    BlockDecryptMut,
    KeyIvInit,
};
use ed25519_dalek::Signer;
use hmac::{
    Hmac,
    Mac,
};
use k256::ecdsa::signature::DigestSigner;
use pkcs8::der::oid::ObjectIdentifier;
use pkcs8::der::{
    Decode,
    Encode,
};
use sec1::EcPrivateKey;
use sha2::Sha512;
use sha3::Digest;
use triomphe::Arc;

use crate::signer::AnySigner;
use crate::{
    AccountId,
    Error,
    PublicKey,
    Transaction,
};

// replace with `array::split_array_ref` when that's stable.
fn split_key_array(arr: &[u8; 64]) -> (&[u8; 32], &[u8; 32]) {
    let (lhs, rhs) = arr.split_at(32);

    // SAFETY: lhs points to [T; N]? Yes it's [T] of length 64/2 (guaranteed by split_at)
    let lhs = unsafe { &*(lhs.as_ptr().cast::<[u8; 32]>()) };
    // SAFETY: rhs points to [T; N]? Yes it's [T] of length 64/2 (rhs.len() = 64 - lhs.len(), lhs.len() has been proven to be 32 above...)
    let rhs = unsafe { &*(rhs.as_ptr().cast::<[u8; 32]>()) };

    (lhs, rhs)
}

pub(super) const ED25519_OID: ObjectIdentifier = ObjectIdentifier::new_unwrap("1.3.101.112");
pub(super) const K256_OID: ObjectIdentifier = ObjectIdentifier::new_unwrap("1.3.132.0.10");

enum PrivateKeyData {
    Ed25519(ed25519_dalek::SigningKey),
    Ecdsa(k256::ecdsa::SigningKey),
}

impl From<ed25519_dalek::SigningKey> for PrivateKeyData {
    fn from(value: ed25519_dalek::SigningKey) -> Self {
        Self::Ed25519(value)
    }
}

impl From<k256::ecdsa::SigningKey> for PrivateKeyData {
    fn from(value: k256::ecdsa::SigningKey) -> Self {
        Self::Ecdsa(value)
    }
}

/// A private key on the Hedera network.
#[derive(Clone)]
pub struct PrivateKey(Arc<PrivateKeyDataWrapper>);

// find a better name
struct PrivateKeyDataWrapper {
    data: PrivateKeyData,
    chain_code: Option<[u8; 32]>,
}

impl PrivateKeyDataWrapper {
    fn new(inner: PrivateKeyData) -> Self {
        Self { data: inner, chain_code: None }
    }

    fn new_derivable(inner: PrivateKeyData, chain_code: [u8; 32]) -> Self {
        Self { data: inner, chain_code: Some(chain_code) }
    }
}

impl From<ed25519_dalek::SigningKey> for PrivateKeyDataWrapper {
    fn from(value: ed25519_dalek::SigningKey) -> Self {
        Self::new(value.into())
    }
}

impl From<k256::ecdsa::SigningKey> for PrivateKeyDataWrapper {
    fn from(value: k256::ecdsa::SigningKey) -> Self {
        Self::new(value.into())
    }
}

// for usage in tests (provides a way to snapshot test)
impl Debug for PrivateKeyDataWrapper {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        #[derive(Debug)]
        enum Algorithm {
            Ed25519,
            Ecdsa,
        }

        let (algorithm, key) = match &self.data {
            PrivateKeyData::Ed25519(key) => (Algorithm::Ed25519, hex::encode(key.to_bytes())),
            PrivateKeyData::Ecdsa(key) => (Algorithm::Ecdsa, hex::encode(key.to_bytes())),
        };

        f.debug_struct("PrivateKeyData")
            .field("algorithm", &algorithm)
            .field("key", &key)
            .field("chain_code", &self.chain_code.as_ref().map(hex::encode))
            .finish()
    }
}

impl PrivateKey {
    #[cfg(test)]
    pub(crate) fn debug_pretty(&self) -> &impl Debug {
        &*self.0
    }

    fn new(data: PrivateKeyDataWrapper) -> Self {
        Self(Arc::new(data))
    }

    fn new_derivable(key: PrivateKeyData, chain_code: [u8; 32]) -> Self {
        Self::new(PrivateKeyDataWrapper::new_derivable(key, chain_code))
    }

    fn ed25519(key: ed25519_dalek::SigningKey) -> Self {
        Self::new(key.into())
    }

    fn ecdsa(key: k256::ecdsa::SigningKey) -> Self {
        Self::new(key.into())
    }

    /// Generates a new Ed25519 `PrivateKey`.
    #[must_use]
    pub fn generate_ed25519() -> Self {
        use rand::Rng as _;

        let mut csprng = rand::thread_rng();

        let data = ed25519_dalek::SigningKey::generate(&mut csprng);

        Self::new_derivable(data.into(), csprng.gen())
    }

    /// Generates a new ECDSA(secp256k1) `PrivateKey`.
    #[must_use]
    pub fn generate_ecdsa() -> Self {
        let data = k256::ecdsa::SigningKey::random(&mut rand::thread_rng());

        Self::ecdsa(data)
    }

    /// Gets the [`PublicKey`] which corresponds to this `PrivateKey`.
    #[must_use]
    pub fn public_key(&self) -> PublicKey {
        match &self.0.data {
            PrivateKeyData::Ed25519(key) => PublicKey::ed25519(key.verifying_key()),
            PrivateKeyData::Ecdsa(key) => PublicKey::ecdsa(*key.verifying_key()),
        }
    }

    /// Parse a `PrivateKey` from a sequence of bytes.
    ///
    /// # Errors
    /// - [`Error::KeyParse`] if `bytes` cannot be parsed into a `PrivateKey`.
    pub fn from_bytes(bytes: &[u8]) -> crate::Result<Self> {
        if bytes.len() == 32 || bytes.len() == 64 {
            return Self::from_bytes_ed25519(bytes);
        }

        Self::from_bytes_der(bytes)
    }

    /// Parse a Ed25519 `PrivateKey` from a sequence of bytes.
    ///
    /// # Errors
    /// - [`Error::KeyParse`] if `bytes` cannot be parsed into a ed25519 `PrivateKey`.
    // panic is unreachable.
    #[allow(clippy::missing_panics_doc)]
    pub fn from_bytes_ed25519(bytes: &[u8]) -> crate::Result<Self> {
        match bytes.len() {
            32 | 64 => Ok(Self::ed25519(ed25519_dalek::SigningKey::from_bytes(
                &bytes[..32].try_into().unwrap(),
            ))),
            _ => Self::from_bytes_der(bytes),
        }
    }

    /// Parse a ECDSA(secp256k1) `PrivateKey` from a sequence of bytes.
    ///
    /// # Errors
    /// - [`Error::KeyParse`] if `bytes` cannot be parsed into a ECDSA(secp256k1) `PrivateKey`.
    pub fn from_bytes_ecdsa(bytes: &[u8]) -> crate::Result<Self> {
        let data = (bytes.len() == 32).then(|| GenericArray::from_slice(bytes));

        let data = if let Some(bytes) = data {
            // not DER encoded, raw bytes for key
            k256::ecdsa::SigningKey::from_bytes(bytes).map_err(Error::key_parse)?
        } else {
            return Self::from_bytes_der(bytes);
        };

        Ok(Self::ecdsa(data))
    }

    /// Parse a `PrivateKey` from a sequence of der encoded bytes.
    ///
    /// # Errors
    /// - [`Error::KeyParse`] if `bytes` cannot be parsed into a `PrivateKey`.
    pub fn from_bytes_der(bytes: &[u8]) -> crate::Result<Self> {
        let info =
            pkcs8::PrivateKeyInfo::from_der(bytes).map_err(|err| Error::key_parse(err.to_string()));

        let info = match info {
            Ok(info) => info,
            Err(e) => return Self::from_sec1_bytes_der(bytes).ok().ok_or(e),
        };

        // PrivateKey is an `OctetString`, and the `PrivateKey`s we all support are `OctetStrings`.
        // So, we, awkwardly, have an `OctetString` containing an `OctetString` containing our key material.
        let inner = pkcs8::der::asn1::OctetStringRef::from_der(info.private_key)
            .map_err(|err| Error::key_parse(err.to_string()))?;

        let inner = inner.as_bytes();

        match info.algorithm.oid {
            K256_OID => Self::from_bytes_ecdsa(inner),
            ED25519_OID => Self::from_bytes_ed25519(inner),
            id => Err(Error::key_parse(format!("unsupported key algorithm: {id}"))),
        }
    }

    fn from_sec1_bytes_der(bytes: &[u8]) -> crate::Result<Self> {
        let sec1 = EcPrivateKey::try_from(bytes).map_err(Error::key_parse)?;

        match sec1.parameters.and_then(sec1::EcParameters::named_curve) {
            Some(K256_OID) => Self::from_bytes_ecdsa(sec1.private_key),
            Some(oid) => Err(Error::key_parse(format!("unsupported curve OID: {oid}"))),
            None => Err(Error::key_parse("missing curve parameters")),
        }
    }

    /// Parse a `PrivateKey` from a der encoded string.
    ///
    /// Optionally strips a `0x` prefix.
    /// See [`from_bytes_der`](Self::from_bytes_der).
    ///
    /// # Errors
    /// - [`Error::KeyParse`] if `s` cannot be parsed into a `PrivateKey`.
    pub fn from_str_der(s: &str) -> crate::Result<Self> {
        Self::from_bytes_der(
            &hex::decode(s.strip_prefix("0x").unwrap_or(s)).map_err(Error::key_parse)?,
        )
    }

    /// Parse a Ed25519 `PrivateKey` from a string containing the raw key material.
    ///
    /// Optionally strips a `0x` prefix.
    /// See: [`from_bytes_ed25519`](Self::from_bytes_ed25519).
    ///
    /// # Errors
    /// - [`Error::KeyParse`] if `s` cannot be parsed into a ed25519 `PrivateKey`.
    pub fn from_str_ed25519(s: &str) -> crate::Result<Self> {
        Self::from_bytes_ed25519(
            &hex::decode(s.strip_prefix("0x").unwrap_or(s)).map_err(Error::key_parse)?,
        )
    }

    /// Parse a ECDSA(secp256k1) `PrivateKey` from a string containing the raw key material.
    ///
    /// Optionally strips a `0x` prefix.
    /// See: [`from_str_ecdsa`](Self::from_str_ecdsa).
    ///
    /// # Errors
    /// - [`Error::KeyParse`] if `s` cannot be parsed into a ECDSA(secp256k1) `PrivateKey`.
    pub fn from_str_ecdsa(s: &str) -> crate::Result<Self> {
        Self::from_bytes_ecdsa(
            &hex::decode(s.strip_prefix("0x").unwrap_or(s)).map_err(Error::key_parse)?,
        )
    }

    /// Parse a `PrivateKey` from [PEM](https://www.rfc-editor.org/rfc/rfc7468#section-10) encoded bytes.
    ///
    /// # Errors
    /// - [`Error::KeyParse`] if `pem` is not valid PEM.
    /// - [`Error::KeyParse`] if the type label (BEGIN XYZ) is not `PRIVATE KEY`.
    /// - [`Error::KeyParse`] if the data contained inside the PEM is not a valid `PrivateKey`.
    pub fn from_pem(pem: impl AsRef<[u8]>) -> crate::Result<Self> {
        fn inner(pem: &[u8]) -> crate::Result<PrivateKey> {
            let pem = ::pem::parse(pem).map_err(Error::key_parse)?;

            let type_label = pem.tag();
            let der = pem.contents();

            match type_label {
                "PRIVATE KEY" => PrivateKey::from_bytes_der(der),
                "EC PRIVATE KEY" => PrivateKey::from_sec1_bytes_der(der),
                _ => Err(Error::key_parse(format!(
                    "incorrect PEM type label: expected: `PRIVATE KEY`, got: `{type_label}`"
                ))),
            }
        }

        inner(pem.as_ref())
    }

    /// Parse a `PrivateKey` from encrypted [PEM](https://www.rfc-editor.org/rfc/rfc7468#section-11) encoded bytes.
    /// # Errors
    /// - [`Error::KeyParse`] if `pem` is not valid PEM.
    /// - [`Error::KeyParse`] if the type label (`BEGIN XYZ`) is not `ENCRYPTED PRIVATE KEY`.
    /// - [`Error::KeyParse`] if decrypting the private key fails.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use hedera::PrivateKey;
    /// use hex_literal::hex;
    ///
    /// // ⚠️ WARNING ⚠️
    /// // don't use this private key in your applications, it is compromised by virtue of being here.
    /// let pem = "-----BEGIN ENCRYPTED PRIVATE KEY-----
    /// MIGbMFcGCSqGSIb3DQEFDTBKMCkGCSqGSIb3DQEFDDAcBAjeB6TNNQX+1gICCAAw
    /// DAYIKoZIhvcNAgkFADAdBglghkgBZQMEAQIEENfMacg1/Txd/LhKkxZtJe0EQEVL
    /// mez3xb+sfUIF3TKEIDJtw7H0xBNlbAfLxTV11pofiar0z1/WRBHFFUuGIYSiKjlU
    /// V9RQhAnemO84zcZfTYs=
    /// -----END ENCRYPTED PRIVATE KEY-----";
    ///
    /// let password = "test";
    ///
    /// let sk = PrivateKey::from_pem_with_password(pem, password)?;
    ///
    /// let expected_signature = hex!(
    ///     "a0e5f7d1cf06a4334be4f856aeb427f7"
    ///     "fd53ea7e5c66f10eaad083d736a5adfd"
    ///     "0ac7e4fd3fa90f6b6aad8f1df4149ecd"
    ///     "330a91d5ebff832b11bf14d43eaf5600"
    /// );
    /// assert_eq!(sk.sign(b"message").as_slice(), expected_signature.as_slice());
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_pem_with_password(
        pem: impl AsRef<[u8]>,
        password: impl AsRef<[u8]>,
    ) -> crate::Result<Self> {
        fn inner(pem: &[u8], password: &[u8]) -> crate::Result<PrivateKey> {
            let pem = ::pem::parse(pem).map_err(Error::key_parse)?;

            let type_label = pem.tag();
            let der = pem.contents();

            match type_label {
                "ENCRYPTED PRIVATE KEY" => {
                    // thanks library for not providing a way to check `is_empty`.
                    if let Some(header) = pem.headers().iter().next() {
                        return Err(Error::key_parse(format!("Expected no headers for `ENCRYPTED PRIVATE KEY` but found: `{}: {}`", header.0, header.1)));
                    }

                    let info = pkcs8::EncryptedPrivateKeyInfo::from_der(der)
                        .map_err(|e| Error::key_parse(e.to_string()))?;

                    let decrypted =
                        info.decrypt(password).map_err(|e| Error::key_parse(e.to_string()))?;

                    PrivateKey::from_bytes_der(decrypted.as_bytes())
                }

                "EC PRIVATE KEY" => {
                    let (headers, mut der) = {
                        let mut pem = pem;
                        let headers = mem::take(pem.headers_mut());
                        let  der = pem.into_contents();

                        (headers, der)
                    };

                    let proc_type = headers.get("Proc-Type");
                    let dek_info = headers.get("DEK-Info");

                    if proc_type != Some("4,ENCRYPTED") {
                        return Err(Error::key_parse("Encrypted EC Private Key missing or invalid `Proc-Type` header"));
                    }

                    let Some(dek_info) = dek_info else {
                        return Err(Error::key_parse("Encrypted EC Private Key missing `DEK-Info` header"));
                    };

                    let Some((alg, iv)) = dek_info.split_once(',') else {
                        return Err(Error::key_parse("Invalid `DEK-Info`"));
                    };

                    let iv = hex::decode(iv).map_err(|e| Error::key_parse(format!("invalid IV: {e}")))?;

                    let decrypted = match alg {
                        "AES-128-CBC" => {
                            let iv: [u8; 16] = iv.try_into().map_err(|_| Error::key_parse("invalid IV"))?;


                            let mut md5_sum = md5::Context::new();

                            md5_sum.consume(password);
                            md5_sum.consume(&iv[..8]);

                            let passphrase = md5_sum.compute().0;

                            cbc::Decryptor::<aes::Aes128>::new(&passphrase.into(), &iv.into())
                            .decrypt_padded_mut::<aes::cipher::block_padding::Pkcs7>(&mut der).map_err(|e| Error::key_parse(format!("error decrypting key: {e}")))?
                        }
                        _ => return Err(Error::key_parse(format!("unexpected decryption alg: {alg}")))
                    };

                    PrivateKey::from_sec1_bytes_der(decrypted)
                }

                _ => Err(Error::key_parse(format!(
                    "incorrect PEM type label: expected: `ENCRYPTED PRIVATE KEY`, got: `{type_label}`"
                )))
            }
        }

        inner(pem.as_ref(), password.as_ref())
    }

    /// Return this `PrivateKey`, serialized as der encoded bytes.
    // panic should be impossible (`unreachable`)
    #[allow(clippy::missing_panics_doc)]
    #[must_use]
    pub fn to_bytes_der(&self) -> Vec<u8> {
        let mut inner = Vec::with_capacity(34);

        pkcs8::der::asn1::OctetStringRef::new(&self.to_bytes_raw_internal())
            .unwrap()
            .encode_to_vec(&mut inner)
            .unwrap();

        let info = pkcs8::PrivateKeyInfo {
            algorithm: self.algorithm(),
            private_key: &inner,
            public_key: None,
        };

        let mut buf = Vec::with_capacity(64);
        info.encode_to_vec(&mut buf).unwrap();

        buf
    }

    /// Return this `PrivateKey`, serialized as bytes.
    ///
    /// If this is an ed25519 private key, this is equivalent to [`to_bytes_raw`](Self::to_bytes_raw)
    /// If this is an ecdsa private key, this is equivalent to [`to_bytes_der`](Self::to_bytes_der)
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        match &self.0.data {
            PrivateKeyData::Ed25519(_) => self.to_bytes_raw(),
            PrivateKeyData::Ecdsa(_) => self.to_bytes_der(),
        }
    }

    /// Return this `PrivateKey`, serialized as bytes.
    #[must_use]
    pub fn to_bytes_raw(&self) -> Vec<u8> {
        self.to_bytes_raw_internal().as_slice().to_vec()
    }

    #[must_use]
    fn to_bytes_raw_internal(&self) -> [u8; 32] {
        match &self.0.data {
            PrivateKeyData::Ed25519(key) => key.to_bytes(),
            PrivateKeyData::Ecdsa(key) => key.to_bytes().into(),
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
        hex::encode(self.to_bytes_raw_internal())
    }

    /// Creates an [`AccountId`] with the given `shard`, `realm`, and `self.public_key()` as an [`alias`](AccountId::alias).
    ///
    /// # Examples
    ///
    /// ```
    /// use hedera::PrivateKey;
    ///
    /// let key: PrivateKey = "3030020100300706052b8104000a042204208776c6b831a1b61ac10dac0304a2843de4716f54b1919bb91a2685d0fe3f3048".parse().unwrap();
    ///
    /// let account_id = key.to_account_id(0, 0);
    /// assert_eq!(account_id.to_string(), "0.0.302d300706052b8104000a03220002703a9370b0443be6ae7c507b0aec81a55e94e4a863b9655360bd65358caa6588");
    /// ```
    #[inline(always)]
    #[must_use]
    pub fn to_account_id(&self, shard: u64, realm: u64) -> AccountId {
        self.public_key().to_account_id(shard, realm)
    }

    fn algorithm(&self) -> pkcs8::AlgorithmIdentifierRef<'_> {
        pkcs8::AlgorithmIdentifierRef {
            parameters: None,
            oid: match &self.0.data {
                PrivateKeyData::Ed25519(_) => ED25519_OID,
                PrivateKeyData::Ecdsa(_) => K256_OID,
            },
        }
    }

    /// Returns `true` if `self` is an Ed25519 `PrivateKey`.
    ///
    /// # Examples
    /// ```
    /// use hedera::PrivateKey;
    /// let sk = PrivateKey::generate_ed25519();
    ///
    /// assert!(sk.is_ed25519());
    /// ```
    /// ```
    /// use hedera::PrivateKey;
    /// let sk = PrivateKey::generate_ecdsa();
    ///
    /// assert!(!sk.is_ed25519());
    /// ```
    #[must_use]
    pub fn is_ed25519(&self) -> bool {
        matches!(self.0.data, PrivateKeyData::Ed25519(_))
    }

    /// Returns `true` if this is an ECDSA(secp256k1) `PrivateKey`.
    ///
    /// # Examples
    /// ```
    /// use hedera::PrivateKey;
    /// let sk = PrivateKey::generate_ecdsa();
    ///
    /// assert!(sk.is_ecdsa());
    /// ```
    /// ```
    /// use hedera::PrivateKey;
    /// let sk = PrivateKey::generate_ed25519();
    ///
    /// assert!(!sk.is_ecdsa());
    /// ```
    #[must_use]
    pub fn is_ecdsa(&self) -> bool {
        matches!(self.0.data, PrivateKeyData::Ecdsa(_))
    }

    /// Signs the given `message`.
    #[must_use]
    pub fn sign(&self, message: &[u8]) -> Vec<u8> {
        match &self.0.data {
            PrivateKeyData::Ed25519(key) => key.sign(message).to_bytes().as_slice().to_vec(),
            PrivateKeyData::Ecdsa(key) => {
                let signature: k256::ecdsa::Signature =
                    key.sign_digest(sha3::Keccak256::new_with_prefix(message));

                signature.to_vec()
            }
        }
    }

    // I question the reason for this function existing.
    /// Signs the given transaction.
    ///
    /// # Errors
    /// This function will freeze the transaction if it is not frozen.
    /// As such, any error that can be occur during [`Transaction::freeze`] can also occur here.
    pub fn sign_transaction<D: crate::transaction::TransactionExecute>(
        &self,
        transaction: &mut Transaction<D>,
    ) -> crate::Result<Vec<u8>> {
        transaction.freeze()?;

        let sig = transaction.add_signature_signer(&AnySigner::PrivateKey(self.clone()));

        Ok(sig)
    }

    /// Returns true if calling [`derive`](Self::derive) on `self` would succeed.
    #[must_use]
    pub fn is_derivable(&self) -> bool {
        self.is_ed25519() && self.0.chain_code.is_some()
    }

    /// Derives a child key based on `index`.
    ///
    /// # Errors
    /// - [`Error::KeyDerive`] if this is an Ecdsa key (unsupported operation)
    /// - [`Error::KeyDerive`] if this key has no `chain_code` (key is not derivable)
    // this is specifically for the two `try_into`s which depend on `split_array_ref`.
    // Any panic would indicate a bug in this crate or a dependency of it, not in user code.
    #[allow(clippy::missing_panics_doc)]
    pub fn derive(&self, index: i32) -> crate::Result<Self> {
        const HARDEND_MASK: u32 = 1 << 31;
        let index = index as u32;

        let chain_code =
            self.0.chain_code.as_ref().ok_or_else(|| Error::key_derive("key is underivable"))?;

        match &self.0.data {
            PrivateKeyData::Ed25519(key) => {
                // force hardened.
                let index = index | HARDEND_MASK;

                let output: [u8; 64] = Hmac::<Sha512>::new_from_slice(chain_code)
                    .expect("HMAC can take keys of any size")
                    .chain_update([0])
                    .chain_update(key.to_bytes())
                    .chain_update(index.to_be_bytes())
                    .finalize()
                    .into_bytes()
                    .into();

                // todo: use `split_array_ref` when that's stable.
                let (data, chain_code) = split_key_array(&output);

                let data = ed25519_dalek::SigningKey::from_bytes(data);

                Ok(Self::new_derivable(data.into(), *chain_code))
            }
            PrivateKeyData::Ecdsa(_) => {
                Err(Error::key_derive("Ecdsa private keys don't currently support derivation"))
            }
        }
    }

    // todo: what do we do about i32?
    // It's basically just a cast to support them, but, unlike Java, operator overloading doesn't exist.
    /// Derive a `PrivateKey` based on `index`.
    ///
    /// # Errors
    /// - [`Error::KeyDerive`] if this is an Ecdsa key (unsupported operation)
    // ⚠️ unaudited cryptography ⚠️
    pub fn legacy_derive(&self, index: i64) -> crate::Result<Self> {
        match &self.0.data {
            PrivateKeyData::Ed25519(key) => {
                let entropy = key.to_bytes();
                let mut seed = Vec::with_capacity(entropy.len() + 8);

                seed.extend_from_slice(&entropy);

                let i1: i32 = match index {
                    0x00ff_ffff_ffff => 0xff,
                    0.. => 0,
                    _ => -1,
                };

                let i2 = index as u8;

                seed.extend_from_slice(&i1.to_be_bytes());
                // any better way to do this?
                seed.extend_from_slice(&[i2; 4]);

                let mat = pbkdf2::pbkdf2_hmac_array::<Sha512, 32>(&seed, &[0xff], 2048);

                Ok(Self::ed25519(ed25519_dalek::SigningKey::from_bytes(&mat)))
            }

            PrivateKeyData::Ecdsa(_) => {
                Err(Error::key_derive("Ecdsa private keys don't currently support derivation"))
            }
        }
    }

    #[cfg(feature = "mnemonic")]
    pub(crate) fn from_mnemonic_seed(seed: &[u8]) -> Self {
        let output: [u8; 64] = Hmac::<Sha512>::new_from_slice(b"ed25519 seed")
            .expect("hmac can take a seed of any size")
            .chain_update(seed)
            .finalize()
            .into_bytes()
            .into();

        // todo: use `split_array_ref` when that's stable.
        let (left, right) = {
            let (left, right) = output.split_at(32);
            let left: [u8; 32] = left.try_into().unwrap();
            let right: [u8; 32] = right.try_into().unwrap();
            (left, right)
        };

        let data = ed25519_dalek::SigningKey::from_bytes(&left);

        let mut key = Self::new_derivable(data.into(), right);

        for index in [44, 3030, 0, 0] {
            key = key.derive(index).expect("BUG: we set the chain code earlier in this function");
        }

        key
    }

    /// Recover a `PrivateKey` from a mnemonic phrase and a passphrase.
    // this is specifically for the two `try_into`s which depend on `split_array_ref`.
    // There *is* a 3rd unwrap for a "key is not derivable" error, but we construct a key that _is_ derivable.
    // Any panic would indicate a bug in this crate or a dependency of it, not in user code.
    #[cfg(feature = "mnemonic")]
    #[allow(clippy::missing_panics_doc)]
    #[must_use]
    pub fn from_mnemonic(mnemonic: &crate::Mnemonic, passphrase: &str) -> Self {
        let seed = mnemonic.to_seed(passphrase);
        Self::from_mnemonic_seed(&seed)
    }

    #[must_use]
    pub(crate) fn _kind(&self) -> super::KeyKind {
        match &self.0.data {
            PrivateKeyData::Ed25519(_) => super::KeyKind::Ed25519,
            PrivateKeyData::Ecdsa(_) => super::KeyKind::Ecdsa,
        }
    }
}

impl Debug for PrivateKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "\"{self}\"")
    }
}

impl Display for PrivateKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.pad(&self.to_string_der())
    }
}

impl FromStr for PrivateKey {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_bytes(&hex::decode(s.strip_prefix("0x").unwrap_or(s)).map_err(Error::key_parse)?)
    }
}

// TODO: derive (!) - secp256k1
// TODO: legacy_derive (!) - secp256k1
