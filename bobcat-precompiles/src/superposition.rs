#![allow(unused)]

use bobcat_maths::U;

use bobcat_cd::address;

use bobcat_call::static_call_unit;

use array_concat::concat_arrays;

use sha2::{digest::Update, Digest, Sha512};

#[cfg(feature = "ed25519-dalek")]
pub use crate::ed25519::const_edphverify;

/// EdVerify is deployed at this address on Arbitrum One and Superposition.
pub const ADDR_EDVERIFY: [u8; 20] = address!(b"c3e443be2cfa4f41a5f5e4978d012847d355b419");

/// Muldiv is deployed at this address on Arbitrum One and Superposition.
pub const ADDR_MUL_DIV: [u8; 20] = address!(b"7a9579a78d6ea3279b33d6d0f92a2fe8fd0e2662");

// Gas for the edverify function assumes the contract is a part of the
// Stylus cache. If it's not, this may fail.
const GAS_EDPHVERIFY: u64 = 96273;

const GAS_MUL_DIV: u64 = 9000;

pub fn const_sha512(x: &[u8]) -> [u8; 64] {
    let mut d = Sha512::new();
    Update::update(&mut d, x);
    d.finalize().into()
}

pub fn sha512(x: &[u8]) -> [u8; 64] {
    const_sha512(x)
}

#[cfg(all(target_family = "wasm", target_os = "unknown"))]
pub fn edphverify_post(digest: [u8; 64], pub_key: U, sig: [u8; 64]) -> bool {
    let cd: [u8; 64 * 2 + 32] = concat_arrays!(digest, pub_key.0, sig);
    static_call_unit(ADDR_EDVERIFY, &cd, GAS_EDPHVERIFY)
}

#[cfg(all(
    not(all(target_family = "wasm", target_os = "unknown")),
    feature = "ed25519-dalek"
))]
pub use const_edphverify as edphverify_post;

#[cfg(any(
    all(target_family = "wasm", target_os = "unknown"),
    feature = "ed25519-dalek"
))]
pub fn edphverify_post_opt(digest: [u8; 64], pub_key: U, sig: [u8; 64]) -> Option<()> {
    if edphverify_post(digest, pub_key, sig) {
        Some(())
    } else {
        None
    }
}

#[cfg(any(
    all(target_family = "wasm", target_os = "unknown"),
    feature = "ed25519-dalek"
))]
pub fn edphverify_pre(pre: &[u8], pub_key: U, sig: [u8; 64]) -> bool {
    edphverify_post(sha512(pre), pub_key, sig)
}

#[cfg(any(
    all(target_family = "wasm", target_os = "unknown"),
    feature = "ed25519-dalek"
))]
pub fn edphverify_pre_opt(pre: &[u8], pub_key: U, sig: [u8; 64]) -> Option<()> {
    if edphverify_pre(pre, pub_key, sig) {
        Some(())
    } else {
        None
    }
}

#[cfg(all(target_family = "wasm", target_os = "unknown"))]
pub fn mul_div(x: U, y: U, z: U) -> bool {
    let cd: [u8; 3 * 32] = concat_arrays!(x.0, y.0, z.0);
    static_call_unit(ADDR_MUL_DIV, &cd, GAS_MUL_DIV)
}

#[cfg(not(all(target_family = "wasm", target_os = "unknown")))]
pub use bobcat_maths::mul_div;
