// SPDX-License-Identifier: Apache-2.0

use hiero_sdk::PrivateKey;

fn main() {
    // Generate a ECDSA(secp256k1) key
    // This is the recommended default for Hiero (standardized as of 2026)

    let private = PrivateKey::generate_ecdsa();
    let public = private.public_key();

    println!("ecdsa(secp256k1) private = {private}");
    println!("ecdsa(secp256k1) public = {public}");

    // Generate a Ed25519 key
    // This is also supported for legacy compatibility

    let private = PrivateKey::generate_ed25519();
    let public = private.public_key();

    println!("ed25519 private = {private}");
    println!("ed25519 public = {public}");
}
