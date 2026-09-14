use bobcat_cd::address;

use bobcat_call::delegate_call_slice;

#[cfg(feature = "alloc")]
use bobcat_call::delegate_call_vec;

use array_concat::concat_arrays;

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

pub const ADDR_RISC_RUNNER: [u8; 20] = address!(b"215dc94d90fa87642def299e0c018829647b50c4");

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
