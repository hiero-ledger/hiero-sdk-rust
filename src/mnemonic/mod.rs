// SPDX-License-Identifier: Apache-2.0

use std::cmp::Ordering;
use std::fmt;
use std::str::FromStr;

use fraction::{
    Integer,
    ToPrimitive,
};
use num_bigint::BigInt;
use once_cell::sync::Lazy;
use rand::{
    thread_rng,
    RngCore,
};
use sha2::Digest;

use crate::error::{
    MnemonicEntropyError,
    MnemonicParseError,
};
use crate::{
    Error,
    PrivateKey,
};

const BIP39: &str = include_str!("bip39-english.txt");
const LEGACY: &str = include_str!("legacy-english.txt");

// `is_sorted` is unstable.
// this is just lifted from the stdlib impl.
fn is_sorted<T: PartialOrd>(vs: &[T]) -> bool {
    vs.windows(2).all(|w| w[0].partial_cmp(&w[1]).map_or(false, |o| o != Ordering::Greater))
}

// sadly can't do this with a const.
static BIP39_WORD_LIST: Lazy<Vec<&'static str>> = Lazy::new(|| {
    let it: Vec<_> = BIP39.split_whitespace().collect();
    // if the word list is sorted we can use the power of `binary_search` which turns the `O(n)` search into a `O(log n)`
    // n is a constant here, but perf is perf.
    assert!(is_sorted(&it));
    it
});

// the legacy word list isn't sorted.
static LEGACY_WORD_LIST: Lazy<Vec<&'static str>> =
    Lazy::new(|| LEGACY.split_whitespace().collect());

///  `BIP-39` 24-word mnemonic phrase compatible with the Android and iOS mobile wallets.
pub struct Mnemonic(MnemonicData);

// pretend to be the API we want to show
impl fmt::Debug for Mnemonic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Mnemonic")
            .field("words", &self.words())
            .field("is_legacy", &self.is_legacy())
            .finish()
    }
}

impl Mnemonic {
    // todo(sr): before release, try to find a better internal representation
    // lets not expose this until we know what the final signature should be
    pub(crate) fn words(&self) -> &[String] {
        match &self.0 {
            MnemonicData::V1(it) => it.words(),
            MnemonicData::V2V3(it) => it.words(),
        }
    }

    /// Returns `true` if `self` is a legacy mnemonic.
    #[must_use]
    pub fn is_legacy(&self) -> bool {
        matches!(&self.0, MnemonicData::V1(_))
    }

    // todo(sr): Not too happy about requiring a `Vec<String>`
    /// Constructs a `Mnemonic` from a 24-word list.
    ///
    /// # Errors
    /// * if the mnemonic has an invalid length.
    /// * if the mnemonic uses invalid words.
    /// * if the mnemonic has an invalid checksum.
    pub fn from_words(words: Vec<String>) -> crate::Result<Self> {
        let words = match words.try_into() {
            Ok(words) => return Ok(Self(MnemonicData::V1(MnemonicV1 { words: Box::new(words) }))),
            Err(words) => words,
        };

        let mnemonic = Self(MnemonicV2V3 { words }.into());

        if mnemonic.words().len() != 12 && mnemonic.words().len() != 24 {
            return Err(Error::MnemonicParse {
                reason: MnemonicParseError::BadLength(mnemonic.words().len()),
                mnemonic,
            });
        }

        let mut word_indecies = Vec::with_capacity(mnemonic.words().len());
        let mut unknown_words = Vec::new();
        for (word_index, word) in mnemonic.words().iter().enumerate() {
            match BIP39_WORD_LIST.binary_search(&&**word) {
                Ok(i) => {
                    word_indecies.push(i as u16);
                }
                // error (word not in list)
                Err(_) => {
                    unknown_words.push(word_index);
                }
            }
        }

        if !unknown_words.is_empty() {
            return Err(Error::MnemonicParse {
                reason: MnemonicParseError::UnknownWords(unknown_words),
                mnemonic,
            });
        }

        let (entropy, actual_checksum) = incecies_to_entropy_and_checksum(&word_indecies);
        let expected_checksum = checksum(&entropy);
        let expected_checksum =
            if mnemonic.words().len() == 12 { expected_checksum & 0xf0 } else { expected_checksum };

        if expected_checksum != actual_checksum {
            // error: checksum mismatch.
            return Err(Error::MnemonicParse {
                reason: MnemonicParseError::ChecksumMismatch {
                    expected: expected_checksum,
                    actual: actual_checksum,
                },
                mnemonic,
            });
        }

        Ok(mnemonic)
    }

    /// Generate a new 12 word `Mnemonic` from the BIP-39 standard English word list.
    #[must_use]
    pub fn generate_12() -> Self {
        Self(MnemonicV2V3::generate_12().into())
    }

    /// Generate a new 24 word `Mnemonic` from the BIP-39 standard English word list.
    #[must_use]
    pub fn generate_24() -> Self {
        Self(MnemonicV2V3::generate_24().into())
    }

    /// Recover a [`PrivateKey`] from this `Mnemonic`.
    ///
    /// # Errors
    /// Under certain circumstances, this function will return a [`Error::MnemonicEntropy`].
    /// - [`MnemonicEntropyError::ChecksumMismatch`] if the computed checksum doesn't match the actual checksum.
    /// - [`MnemonicEntropyError::BadLength`] if this is a v2 legacy mnemonic and doesn't have `24` words.
    pub fn to_legacy_private_key(&self) -> crate::Result<PrivateKey> {
        let entropy = match &self.0 {
            MnemonicData::V1(it) => it.to_entropy()?,
            MnemonicData::V2V3(it) => it.to_legacy_entropy()?,
        };

        PrivateKey::from_bytes(&entropy)
    }

    /// Recover a [`PrivateKey`] from this `Mnemonic`.
    ///
    /// # Errors
    /// Under certain circumstances, this function will return a [`Error::MnemonicEntropy`].
    /// - [`MnemonicEntropyError::LegacyWithPassphrase`] if this is a legacy private key, and the passphrase isn't empty.
    /// - [`MnemonicEntropyError::ChecksumMismatch`] if this is a legacy private key,
    ///   and the `Mnemonic`'s checksum doesn't match up with the computed one.
    // the only panic here should be unreachable, if it isn't, that's a library bug.
    #[allow(clippy::missing_panics_doc)]
    pub fn to_private_key(&self, passphrase: &str) -> crate::Result<PrivateKey> {
        match &self.0 {
            MnemonicData::V1(_) if !passphrase.is_empty() => {
                Err(Error::from(MnemonicEntropyError::LegacyWithPassphrase))
            }
            MnemonicData::V1(it) => Ok(PrivateKey::from_bytes(&it.to_entropy()?).expect(
                "BUG: invariant broken - V1 mnemonic should always have exactly enough entropy",
            )),
            // known unfixable bug: `PrivateKey::from_mnemonic` can be called with a legacy private key.
            MnemonicData::V2V3(_) => Ok(PrivateKey::from_mnemonic(self, passphrase)),
        }
    }

    pub(crate) fn to_seed(&self, phrase: &str) -> [u8; 64] {
        let mut salt = String::from("mnemonic");
        salt.push_str(phrase);

        pbkdf2::pbkdf2_hmac_array::<sha2::Sha512, 64>(
            self.to_string().as_bytes(),
            salt.as_bytes(),
            2048,
        )
    }
}

impl fmt::Display for Mnemonic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some((first, rest)) = self.words().split_first() {
            f.write_str(first)?;

            for word in rest {
                write!(f, " {word}")?;
            }
        }

        Ok(())
    }
}

impl FromStr for Mnemonic {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_words(s.split_whitespace().map(str::to_owned).collect())
    }
}

struct MnemonicV1 {
    words: Box<[String; 22]>,
}

impl MnemonicV1 {
    // clippy bug.
    #[allow(clippy::explicit_auto_deref)]
    fn words(&self) -> &[String] {
        &*self.words
    }

    fn to_entropy(&self) -> crate::Result<Vec<u8>> {
        let indecies = self.words.iter().map(|w| {
            LEGACY_WORD_LIST
                .iter()
                .enumerate()
                .find_map(|(idx, w2)| (w == w2).then_some(idx))
                .map_or(-1, |it| it as i32)
        });

        let data = convert_radix(indecies, 4096, 256, 33);
        let mut data: Vec<_> = data.into_iter().map(|it| it as u8).collect();

        let (crc, data) = data.split_last_mut().unwrap();

        for item in &mut *data {
            *item ^= *crc;
        }

        let crc2 = crc8(data);
        // checksum mismatch
        if *crc != crc2 {
            return Err(Error::from(MnemonicEntropyError::ChecksumMismatch {
                expected: crc2,
                actual: *crc,
            }));
        }

        Ok(data.to_vec())
    }
}

struct MnemonicV2V3 {
    words: Vec<String>,
}

impl MnemonicV2V3 {
    fn words(&self) -> &[String] {
        &self.words
    }

    fn from_entropy(entropy: &[u8]) -> Self {
        assert!(entropy.len() == 16 || entropy.len() == 32);

        let entropy = {
            let mut it = Vec::with_capacity(entropy.len() + 1);
            it.extend_from_slice(entropy);
            let checksum = checksum(entropy);
            it.push(if entropy.len() == 16 { checksum & 0xf0 } else { checksum });
            it
        };

        let mut buffer = 0_u32;
        let mut offset: u8 = 0;

        let mut words = Vec::with_capacity((entropy.len() * 8 + 1) / 11);

        for byte in entropy {
            buffer = (buffer << 8) | u32::from(byte);
            offset += 8;
            if offset >= 11 {
                let index = (buffer >> (offset - 11) & 0x7ff) as usize;
                words.push(BIP39_WORD_LIST[index].to_owned());
                offset -= 11;
            }
        }

        Self { words }
    }

    fn generate_12() -> Self {
        let mut rng = thread_rng();
        let mut entropy = [0; 16];
        rng.fill_bytes(&mut entropy);

        Self::from_entropy(&entropy)
    }

    fn generate_24() -> Self {
        let mut rng = thread_rng();
        let mut entropy = [0; 32];
        rng.fill_bytes(&mut entropy);

        Self::from_entropy(&entropy)
    }

    fn to_legacy_entropy(&self) -> crate::Result<Vec<u8>> {
        // error here where we'll have more context than `PrivateKey::from_bytes`.
        if self.words.len() != 24 {
            return Err(Error::from(MnemonicEntropyError::BadLength {
                expected: 24,
                actual: self.words.len(),
            }));
        }

        // technically, this code all works for 12 words, but I'm going to pretend it doesn't.
        let (entropy, actual_checksum) = words_to_entropy_and_checksum(&self.words);

        let expected_checksum = checksum(&entropy);
        let expected_checksum =
            if self.words.len() == 12 { expected_checksum & 0xf0 } else { expected_checksum };

        if expected_checksum != actual_checksum {
            return Err(Error::from(MnemonicEntropyError::ChecksumMismatch {
                expected: expected_checksum,
                actual: actual_checksum,
            }));
        }

        Ok(entropy)
    }
}

enum MnemonicData {
    V1(MnemonicV1),
    V2V3(MnemonicV2V3),
}

impl From<MnemonicV1> for MnemonicData {
    fn from(v: MnemonicV1) -> Self {
        Self::V1(v)
    }
}

impl From<MnemonicV2V3> for MnemonicData {
    fn from(v: MnemonicV2V3) -> Self {
        Self::V2V3(v)
    }
}

fn crc8(data: &[u8]) -> u8 {
    let mut crc = 0xff;

    for &it in &data[..(data.len() - 1)] {
        crc ^= it;
        for _ in 0..8 {
            crc = (crc >> 1) ^ if (crc & 1) == 0 { 0 } else { 0xb2 };
        }
    }

    crc ^ 0xff
}

// forgive me, universe, for the crimes I'm about to commit.
//
// todo: this is only done with one base pair and it's 4096->256, maybe there's a much nicer way to do this?
fn convert_radix<I: IntoIterator<Item = i32>>(
    nums: I,
    from_radix: i32,
    to_radix: i32,
    to_length: usize,
) -> Vec<i32> {
    let mut buf = BigInt::from(0);
    let from_radix = BigInt::from(i64::from(from_radix));

    for num in nums {
        buf *= &from_radix;
        buf += num;
    }

    let mut out = vec![0; to_length];

    let to_radix = BigInt::from(i64::from(to_radix));

    for out in out.iter_mut().rev() {
        let rem;
        (buf, rem) = buf.div_rem(&to_radix);
        *out = rem.to_i32().unwrap();
    }

    out
}

fn words_to_entropy_and_checksum<T: AsRef<str>>(words: &[T]) -> (Vec<u8>, u8) {
    let indecies: Vec<_> = words
        .iter()
        .map(T::as_ref)
        .map(|it| BIP39_WORD_LIST.binary_search(&it).unwrap() as u16)
        .collect();

    incecies_to_entropy_and_checksum(&indecies)
}

fn incecies_to_entropy_and_checksum(indecies: &[u16]) -> (Vec<u8>, u8) {
    assert!(matches!(indecies.len(), 12 | 24));

    let mut output = Vec::with_capacity(if indecies.len() == 12 { 17 } else { 33 });
    let mut buf = 0_u32;
    let mut offset: u8 = 0;

    for index in indecies {
        assert!(*index <= 0x7ff);

        buf = (buf << 11) | u32::from(*index);
        offset += 11;
        while offset >= 8 {
            // we want to truncate.
            let byte = (buf >> (offset - 8)) as u8;
            output.push(byte);
            offset -= 8;
        }
    }

    if offset != 0 {
        output.push((buf << offset) as u8);
    }

    let checksum = output.pop().unwrap();
    let checksum = if indecies.len() == 12 { checksum & 0xf0 } else { checksum };
    (output, checksum)
}

fn checksum(bytes: &[u8]) -> u8 {
    assert!(bytes.len() <= 32);
    let checksum = sha2::Sha256::digest(bytes);
    checksum[0]
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use assert_matches::assert_matches;
    use expect_test::expect;
    use hex_literal::hex;

    use super::Mnemonic;
    use crate::error::MnemonicParseError;
    use crate::Error;

    const KNOWN_GOOD_MNEMONICS: &[&str] = &[
        "inmate flip alley wear offer often piece magnet surge toddler submit right radio absent pear floor belt raven price stove replace reduce plate home",
        "tiny denial casual grass skull spare awkward indoor ethics dash enough flavor good daughter early hard rug staff capable swallow raise flavor empty angle",
        "ramp april job flavor surround pyramid fish sea good know blame gate village viable include mixed term draft among monitor swear swing novel track",
        "evoke rich bicycle fire promote climb zero squeeze little spoil slight damage"
    ];

    #[test]
    fn from_string() {
        for s in KNOWN_GOOD_MNEMONICS {
            assert_matches!(Mnemonic::from_str(s), Ok(_))
        }
    }

    #[test]
    fn error_invalid_length() {
        // we can't test for up to `usize` length, but we can test several lengths to be modestly sure.
        // it might seem that testing many lengths would be slow.
        // we test:

        // todo: this feels overengineered.
        // every length up to (and including `DENSE_LIMIT`).
        // arbitrarily chosen to be 48.
        const DENSE_LIMIT: usize = 48;

        let dense_lengths = 0..=DENSE_LIMIT;
        let sparse_lengths = (0..=10).map(|it| it * 12).skip_while(|&it| it <= DENSE_LIMIT);

        for length in dense_lengths.chain(sparse_lengths) {
            if matches!(length, 12 | 22 | 24) {
                continue;
            }

            // this is a word that's explicitly in the word list,
            // to ensure we aren't accidentally testing that this error happens before "word(s) not in list"
            let words = std::iter::repeat("apple".to_owned()).take(length).collect();

            let reason = assert_matches!(Mnemonic::from_words(words), Err(Error::MnemonicParse { reason, .. }) => reason);
            let reported_length = assert_matches!(reason, MnemonicParseError::BadLength(reported_length) => reported_length);

            assert_eq!(reported_length, length);
        }
    }

    #[test]
    fn unknown_words_1() {
        // probably fails on checksum, doesn't matter.
        const MNEMONIC: &str = concat!(
            "obvious favorite remain caution ",
            "remove laptop base vacant ",
            "alone fever slush dune"
        );

        // replace words in `MNEMONIC` one at a time.
        for i in 0..12 {
            let mut words: Vec<_> = MNEMONIC.split_whitespace().map(str::to_owned).collect();
            words[i] = "lorum".to_owned();

            let reason = assert_matches!(Mnemonic::from_words(words), Err(Error::MnemonicParse { reason, .. }) => reason);
            let reported_words = assert_matches!(reason, MnemonicParseError::UnknownWords(reported_words) => reported_words);

            assert_eq!(reported_words, vec![i]);
        }
    }

    #[test]
    fn unknown_words_2() {
        // a 24 word mnemonic containing the following typos:
        // absorb -> adsorb
        // account -> acount
        // acquire -> acquired
        const MNEMONIC: &str = concat!(
            "abandon ability able about above absent ",
            "adsorb abstract absurd abuse access accident ",
            "acount accuse achieve acid acoustic acquired ",
            "across act action actor actress actual"
        );

        let reason = assert_matches!(Mnemonic::from_str(MNEMONIC), Err(Error::MnemonicParse { reason, .. }) => reason);
        let reported_words = assert_matches!(reason, MnemonicParseError::UnknownWords(reported_words) => reported_words);

        assert_eq!(reported_words, vec![6, 12, 17]);
    }

    #[test]
    fn checksum_mismatch_1() {
        const MNEMONIC: &str = concat!(
            "abandon ability able about above absent ",
            "absorb abstract absurd abuse access accident ",
            "account accuse achieve acid acoustic acquire ",
            "across act action actor actress actual"
        );

        let reason = assert_matches!(Mnemonic::from_str(MNEMONIC), Err(Error::MnemonicParse { reason, .. }) => reason);
        let (expected, actual) = assert_matches!(reason, MnemonicParseError::ChecksumMismatch { expected, actual } => (expected, actual));

        assert_eq!(expected, 0xba);
        assert_eq!(actual, 0x17);
    }

    #[test]
    fn checksum_mismatch_2() {
        const MNEMONIC: &str =
            "abandon ability able about above absent absorb abstract absurd abuse access accident";

        let reason = assert_matches!(Mnemonic::from_str(MNEMONIC), Err(Error::MnemonicParse { reason, .. }) => reason);
        let (expected, actual) = assert_matches!(reason, MnemonicParseError::ChecksumMismatch { expected, actual } => (expected, actual));

        assert_eq!(expected, 0x10);
        assert_eq!(actual, 0xb0);
    }

    // inverse of the `from_string` test.
    #[test]
    fn from_entropy() {
        const ENTROPY: &[&[u8]] = &[
            &hex!("744b201a7c399733691c2fda5c6f605ceb0c016882cb14f64ea9eb5b6d68298b"),
            &hex!("e2674c8eb2fcada0c433984da6f52bac56466f914b49bd1a8087ed8b12b15248"),
            &hex!("b1615de02c5da95e15ee0f646f7c5cb02f41e69c9c71df683c1fc78db9b825c7"),
            &hex!("4e172857ab9ac2563fee9c829a4b2e9b"),
        ];

        for (entropy, s) in ENTROPY.iter().zip(KNOWN_GOOD_MNEMONICS) {
            let mnemonic = Mnemonic(super::MnemonicV2V3::from_entropy(entropy).into());

            assert_eq!(&mnemonic.to_string(), s);
        }
    }

    #[test]
    fn mnemonic_3() {
        // rustfmt does *not* like long strings.
        const MNEMONIC: &str = concat!(
            "obvious favorite remain caution ",
            "remove laptop base vacant ",
            "increase video erase pass ",
            "sniff sausage knock grid ",
            "argue salt romance way ",
            "alone fever slush dune"
        );

        let mnemonic = Mnemonic::from_str(MNEMONIC).unwrap();
        let key = mnemonic.to_legacy_private_key().unwrap();

        // skip the derives and just test the key.
        // (bugs in `legacy_derive` shouldn't make this function fail.)
        expect![[r#"
            PrivateKeyData {
                algorithm: Ed25519,
                key: "98aa82d6125b5efa04bf8372be7931d05cd77f5ef3330b97d6ee7c006eaaf312",
                chain_code: None,
            }
        "#]]
        .assert_debug_eq(key.debug_pretty())
    }

    #[test]
    fn legacy_mnemonic() {
        const MNEMONIC: &str = concat!(
            "jolly kidnap tom lawn drunk chick optic lust mutter mole bride ",
            "galley dense member sage neural widow decide curb aboard margin manure"
        );

        let mnemonic = Mnemonic::from_str(MNEMONIC).unwrap();

        let key = mnemonic.to_legacy_private_key().unwrap();

        expect![[r#"
            PrivateKeyData {
                algorithm: Ed25519,
                key: "00c2f59212cb3417f0ee0d38e7bd876810d04f2dd2cb5c2d8f26ff406573f2bd",
                chain_code: None,
            }
        "#]]
        .assert_debug_eq(key.debug_pretty());
    }

    #[test]
    fn to_private_key() {
        const MNEMONIC: &str = concat!(
            "inmate flip alley wear offer often ",
            "piece magnet surge toddler submit right ",
            "radio absent pear floor belt raven ",
            "price stove replace reduce plate home"
        );

        let mnemonic = Mnemonic::from_str(MNEMONIC).unwrap();

        let key = mnemonic.to_private_key("").unwrap();

        expect![[r#"
            PrivateKeyData {
                algorithm: Ed25519,
                key: "853f15aecd22706b105da1d709b4ac05b4906170c2b9c7495dff9af49e1391da",
                chain_code: Some(
                    "eb001273d3d54073c42a32c17178d00677e8420631716cd57814cad9db0e64fc",
                ),
            }
        "#]]
        .assert_debug_eq(key.debug_pretty());
    }
}
