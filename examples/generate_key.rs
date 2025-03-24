// SPDX-License-Identifier: Apache-2.0

use hedera::PrivateKey;

fn main() {
    // Generate a Ed25519 key
    // This is the current recommended default for Hedera

    let private = PrivateKey::generate_ed25519();
    let public = private.public_key();

    println!("ed25519 private = {private}");
    println!("ed25519 public = {public}");

    // Generate a ECDSA(secp256k1) key
    // This is recommended for better compatibility with Ethereum

    let private = PrivateKey::generate_ecdsa();
    let public = private.public_key();

    println!("ecdsa(secp256k1) private = {private}");
    println!("ecdsa(secp256k1) public = {public}");
}
