// SPDX-License-Identifier: Apache-2.0

use std::str::FromStr;

use assert_matches::assert_matches;
use expect_test::expect;
use hex_literal::hex;

use crate::{
    EvmAddress,
    PrivateKey,
    PublicKey,
};

#[test]
fn ed25519_from_str() {
    const PK: &str =
        "302a300506032b6570032100e0c8ec2758a5879ffac226a13c0c516b799e72e35141a0dd828f94d37988a4b7";

    let pk = PublicKey::from_str(PK).unwrap();

    assert_eq!(PK, &pk.to_string())
}

#[test]
fn ecdsa_from_str() {
    const PK: &str = "302d300706052b8104000a03220002703a9370b0443be6ae7c507b0aec81a55e94e4a863b9655360bd65358caa6588";

    let pk = PublicKey::from_str(PK).unwrap();

    assert_eq!(PK, &pk.to_string());
}

#[track_caller]
fn pk_from_str_variants(key: &str) {
    // a very low tech solution, but it works!
    for case in 0..4 {
        let prefix = case & 1 == 0;
        let uppercase = (case >> 1) & 1 == 1;
        let prefix = if prefix { "0x" } else { "" };
        let pk = if uppercase { key.to_uppercase() } else { key.to_lowercase() };

        let pk = format!("{prefix}{pk}");
        let pk = PublicKey::from_str(&pk).unwrap();

        assert_eq!(key, &pk.to_string())
    }
}

#[test]
fn ed25519_from_str_variants() {
    pk_from_str_variants(
        "302a300506032b6570032100e0c8ec2758a5879ffac226a13c0c516b799e72e35141a0dd828f94d37988a4b7",
    );
}

#[test]
fn ecdsa_from_str_variants() {
    pk_from_str_variants("302d300706052b8104000a03220002703a9370b0443be6ae7c507b0aec81a55e94e4a863b9655360bd65358caa6588");
}

// copied from Java SDK to ensure conformance.
#[test]
fn to_evm_address() {
    let key = PrivateKey::from_str_ecdsa(
        "debae3ca62ab3157110dba79c8de26540dc320ee9be73a77d70ba175643a3500",
    )
    .unwrap()
    .public_key();

    let evm_address = key.to_evm_address().unwrap();

    assert_eq!(evm_address, EvmAddress(hex!("d8eb8db03c699faa3f47adcdcd2ae91773b10f8b")));
}

#[test]
fn to_evm_address_2() {
    let key = PublicKey::from_str_ecdsa(
        "029469a657510f3bf199a0e29b21e11e7039d8883f3547d59c3568f9c89f704cbc",
    )
    .unwrap();
    let evm_address = key.to_evm_address().unwrap();

    assert_eq!(evm_address, EvmAddress(hex!("bbaa6bdfe888ae1fc8e7c8cee82081fa79ba8834")));
}

#[test]
fn ed25519_verify() {
    let pk = PublicKey::from_str(
        "302a300506032b6570032100e0c8ec2758a5879ffac226a13c0c516b799e72e35141a0dd828f94d37988a4b7",
    )
    .unwrap();

    let signature = hex!(
        "9d04bfed7baa97c80d29a6ae48c0d896ce8463a7ea0c16197d55a563c73996ef"
        "062b2adf507f416c108422c0310fc6fb21886e11ce3de3e951d7a56049743f07"
    );

    pk.verify(b"hello, world", &signature).unwrap();
}

#[test]
fn ecdsa_verify() {
    let pk = PublicKey::from_str(
  "302d300706052b8104000a03220002703a9370b0443be6ae7c507b0aec81a55e94e4a863b9655360bd65358caa6588"
  )
  .unwrap();

    // notice that this doesn't match other impls
    // this is to avoid signature malleability.
    // see: https://github.com/bitcoin/bips/blob/43da5dec5eaf0d8194baa66ba3dd976f923f9d07/bip-0032.mediawiki
    let signature = hex!(
        "f3a13a555f1f8cd6532716b8f388bd4e9d8ed0b252743e923114c0c6cbfe414c"
        "086e3717a6502c3edff6130d34df252fb94b6f662d0cd27e2110903320563851"
    );

    pk.verify(b"hello world", &signature).unwrap();
}

#[test]
fn ed25519_verify_bad_signature() {
    let pk = PublicKey::from_str(
        "302a300506032b6570032100e0c8ec2758a5879ffac226a13c0c516b799e72e35141a0dd828f94d37988a4b7",
    )
    .unwrap();

    // panic!("key: {}", pk.to_string_der());

    let signature = hex!(
        "9d04bfed7baa97c80d29a6ae48c0d896ce8463a7ea0c16197d55a563c73996ef"
        "062b2adf507f416c108422c0310fc6fb21886e11ce3de3e951d7a56049743f00"
    );

    let err = assert_matches!(pk.verify(b"hello, world", &signature), Err(e) => e);

    expect![
        "failed to verify a signature: signature error: Verification equation was not satisfied"
    ]
    .assert_eq(&err.to_string());
}

#[test]
fn ecdsa_verify_bad_signature() {
    let pk = PublicKey::from_str(
        "302d300706052b8104000a03220002703a9370b0443be6ae7c507b0aec81a55e94e4a863b9655360bd65358caa6588",
    )
    .unwrap();

    let signature = hex!(
        "f3a13a555f1f8cd6532716b8f388bd4e9d8ed0b252743e923114c0c6cbfe414c"
        "086e3717a6502c3edff6130d34df252fb94b6f662d0cd27e2110903320563850"
    );

    let err = assert_matches!(pk.verify(b"hello world", &signature), Err(e) => e);

    expect!["failed to verify a signature: signature error"].assert_eq(&err.to_string());
}

#[test]
fn ed25519_verify_error_ecdsa() {
    let pk = PublicKey::from_str(
        "302a300506032b6570032100e0c8ec2758a5879ffac226a13c0c516b799e72e35141a0dd828f94d37988a4b7",
    )
    .unwrap();

    let signature = hex!(
        "f3a13a555f1f8cd6532716b8f388bd4e9d8ed0b252743e923114c0c6cbfe414c"
        "086e3717a6502c3edff6130d34df252fb94b6f662d0cd27e2110903320563851"
    );

    let err = assert_matches!(pk.verify(b"hello, world", &signature), Err(e) => e);

    expect!["failed to verify a signature: signature error: Cannot use scalar with high-bit set"]
        .assert_eq(&err.to_string());
}

#[test]
fn ecdsa_verify_error_ed25519() {
    let pk = PublicKey::from_str(
  "302d300706052b8104000a03220002703a9370b0443be6ae7c507b0aec81a55e94e4a863b9655360bd65358caa6588"
  )
  .unwrap();

    let signature = hex!(
        "9d04bfed7baa97c80d29a6ae48c0d896ce8463a7ea0c16197d55a563c73996ef"
        "062b2adf507f416c108422c0310fc6fb21886e11ce3de3e951d7a56049743f07"
    );

    let err = assert_matches!(pk.verify(b"hello world", &signature), Err(e) => e);

    expect!["failed to verify a signature: signature error"].assert_eq(&err.to_string());
}

#[test]
fn k256_compressed_pkcs8_ec_spki_der() {
    let pk = PublicKey::from_str("3036301006072a8648ce3d020106052b8104000a032200036843f5cb338bbb4cdb21b0da4ea739d910951d6e8a5f703d313efe31afe788f4").unwrap();

    assert_eq!(
        pk.to_string_raw(),
        "036843f5cb338bbb4cdb21b0da4ea739d910951d6e8a5f703d313efe31afe788f4"
    )
}

#[test]
fn k256_uncompressed_pkcs8_ec_spki_der() {
    let pk = PublicKey::from_str("3056301006072a8648ce3d020106052b8104000a03420004aaac1c3ac1bea0245b8e00ce1e2018f9eab61b6331fbef7266f2287750a6597795f855ddcad2377e22259d1fcb4e0f1d35e8f2056300c15070bcbfce3759cc9d").unwrap();

    assert_eq!(
        pk.to_string_raw(),
        "03aaac1c3ac1bea0245b8e00ce1e2018f9eab61b6331fbef7266f2287750a65977"
    )
}
