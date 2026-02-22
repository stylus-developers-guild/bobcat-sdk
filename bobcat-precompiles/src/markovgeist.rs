use bobcat_cd::address;

use bobcat_call::delegate_call_slice;

#[cfg(not(feature = "tickmath-local"))]
use bobcat_cd::const_keccak_sel;

#[cfg(not(feature = "tickmath-local"))]
use bobcat_call::{static_call_slice, static_call_word};

#[cfg(feature = "alloc")]
use bobcat_call::delegate_call_vec;

use array_concat::concat_arrays;

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

use crate::U;

#[cfg(feature = "tickmath-local")]
use markovgeist_precompiles_tickmath as tickmath;

#[cfg(feature = "tickmath-local")]
pub use tickmath::{U160, U256};

pub const ADDR_RISC_RUNNER: [u8; 20] = address!(b"215dc94d90fa87642def299e0c018829647b50c4");

pub const ADDR_TICKMATH: [u8; 20] = address!(b"4e0b6ad0f26f2fa689ac4c808fc50a0b4f8ef126");

#[cfg(not(feature = "tickmath-local"))]
const SEL_PRICE_TO_TICK: [u8; 4] = const_keccak_sel(b"tickAtSqrtRatio(uint256)");

#[cfg(not(feature = "tickmath-local"))]
const SEL_TICK_TO_PRICE: [u8; 4] = const_keccak_sel(b"sqrtRatioAtTick(int32)");

#[cfg(not(feature = "tickmath-local"))]
const SEL_TICK_TO_PRICE_LIMBS: [u8; 4] = const_keccak_sel(b"sqrtRatioAtTickLimbs(int32)");

/// Delegatecall to Orderbookkit's riscv32im runner
/// with a seccomp-style jail. Features can be used to support some
/// application-specific operations in the machine.
pub fn delegate_riscv32im<const CD: usize, const CD_ALL: usize, const CAP: usize>(
    addr: [u8; 20],
    featureset: u16,
    cd: &[u8; CD],
    gas: u64,
) -> (bool, usize, [u8; CAP]) {
    assert_eq!(20 + size_of::<u16>() + CD, CD_ALL, "cap not consistent");
    let args: [u8; CD_ALL] = concat_arrays!(addr, featureset.to_be_bytes(), *cd);
    delegate_call_slice(ADDR_RISC_RUNNER, &args, gas, 0)
}

#[cfg(feature = "alloc")]
pub fn delegate_riscv32im_vec<const CD: usize, const CD_ALL: usize>(
    addr: [u8; 20],
    featureset: u16,
    cd: &[u8; CD],
    gas: u64,
) -> (bool, Vec<u8>) {
    assert_eq!(20 + size_of::<u16>() + CD, CD_ALL, "cap not consistent");
    let args: [u8; CD_ALL] = concat_arrays!(addr, featureset.to_be_bytes(), *cd);
    delegate_call_vec(ADDR_RISC_RUNNER, &args, gas, 0)
}

#[cfg(feature = "alloc")]
pub fn delegate_riscv32im_vec_vec(
    addr: [u8; 20],
    featureset: u16,
    mut cd: Vec<u8>,
    gas: u64,
) -> (bool, Vec<u8>) {
    let args: [u8; 20 + size_of::<u16>()] = concat_arrays!(addr, featureset.to_be_bytes());
    let mut args = args.to_vec();
    args.append(&mut cd);
    delegate_call_vec(ADDR_RISC_RUNNER, &args, gas, 0)
}

#[cfg(all(target_arch = "wasm32", not(feature = "tickmath-local")))]
pub fn price_to_tick(price: [u8; 24]) -> (bool, i32) {
    let b: [u8; 4 + 32 - 24 + 24] = concat_arrays!(SEL_PRICE_TO_TICK, [0u8; 32 - 24], price);
    let (rc, rd) = static_call_word(ADDR_TICKMATH, &b, u64::MAX, 0);
    if !rc {
        (false, 0)
    } else {
        let w: u32 = rd.into();
        (true, w as i32)
    }
}

#[cfg(feature = "tickmath-local")]
pub fn price_to_tick(price: [u8; 24]) -> (bool, i32) {
    match tickmath::price_to_tick(U256::from_be_bytes(price)) {
        Some(v) => (true, v),
        None => (false, 0),
    }
}

#[cfg(all(target_arch = "wasm32", not(feature = "tickmath-local")))]
pub fn tick_to_price(tick: i32) -> (bool, U) {
    let b: [u8; 4 + 32 - 4 + 24] =
        concat_arrays!(SEL_TICK_TO_PRICE, [0u8; 32 - 4], tick.to_be_bytes());
    let (rc, rd) = static_call_word(ADDR_TICKMATH, &b, u64::MAX, 0);
    if !rc {
        (false, U::ZERO)
    } else {
        (true, rd)
    }
}

#[cfg(feature = "tickmath-local")]
pub fn tick_to_price(tick: i32) -> (bool, U) {
    match tickmath::tick_to_price(tick) {
        Some(v) => (true, U(v.to_be_bytes())),
        None => (false, U::ZERO),
    }
}

#[cfg(all(target_arch = "wasm32", not(feature = "tickmath-local")))]
pub fn tick_to_price_limbs(tick: i32) -> (bool, u64, u64, u64) {
    let b: [u8; 4 + 32 - 4 + 24] =
        concat_arrays!(SEL_TICK_TO_PRICE_LIMBS, [0u8; 32 - 4], tick.to_be_bytes());
    let (rc, _, rd) = static_call_slice::<{ size_of::<u64>() * 3 }>(ADDR_TICKMATH, &b, u64::MAX, 0);
    if !rc {
        (false, 0, 0, 0)
    } else {
        let x = u64::from_be_bytes(rd[..8].try_into().unwrap());
        let y = u64::from_be_bytes(rd[8..16].try_into().unwrap());
        let z = u64::from_be_bytes(rd[16..].try_into().unwrap());
        (true, x, y, z)
    }
}

#[cfg(feature = "tickmath-local")]
pub fn tick_to_price_limbs(tick: i32) -> (bool, [u64; 3]) {
    match tickmath::tick_to_price_limbs(tick) {
        Some(xs) => (true, xs),
        None => (false, [0u64; 3]),
    }
}

#[cfg(feature = "tickmath-local")]
pub fn tick_to_price_u160(tick: i32) -> (bool, U160) {
    match tickmath::tick_to_price_limbs(tick) {
        Some(xs) => (true, U160::from_limbs(xs)),
        None => (false, U160::ZERO),
    }
}
