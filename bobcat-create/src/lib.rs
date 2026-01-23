#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

use bobcat_storage::{const_keccak256, keccak256};

pub use bobcat_maths::U;

type Address = [u8; 20];

use array_concat::concat_arrays;

#[cfg(all(target_family = "wasm", target_os = "unknown"))]
use bobcat_host as impls;

#[cfg(not(all(target_family = "wasm", target_os = "unknown")))]
mod impls {
    // Sorry -- on the host, these don't do anything.

    pub(crate) unsafe fn create1(
        _code: *const u8,
        _code_len: usize,
        _endowment: *const u8,
        _contract: *mut u8,
        _revert_data_len: *mut usize,
    ) {
    }

    pub(crate) unsafe fn create2(
        _code: *const u8,
        _code_len: usize,
        _endowment: *const u8,
        _salt: *const u8,
        _contract: *mut u8,
        _revert_data_len: *mut usize,
    ) {
    }

    pub(crate) unsafe fn read_return_data(_dest: *mut u8, _offset: usize, _size: usize) -> usize {
        0
    }
}

pub fn create1_partial(code: &[u8], endowment: U) -> (Address, usize) {
    let mut addr = [0u8; 20];
    let mut revert_len = 0;
    unsafe {
        impls::create1(
            code.as_ptr(),
            code.len(),
            endowment.as_ptr(),
            addr.as_mut_ptr(),
            &mut revert_len as *mut usize,
        )
    }
    (addr, revert_len)
}

pub fn create1_unit(code: &[u8], endowment: U) -> Option<Address> {
    let (addr, _) = create1_partial(code, endowment);
    if addr == [0u8; 20] {
        return None;
    }
    Some(addr)
}

pub fn create1_slice<const REVERT_CAP: usize>(
    code: &[u8],
    endowment: U,
) -> (Address, [u8; REVERT_CAP], usize) {
    let (addr, i) = create1_partial(code, endowment);
    let mut b = [0u8; REVERT_CAP];
    let mut l = 0;
    if addr != [0u8; 20] {
        assert!(REVERT_CAP >= i, "create1 not enough space");
        l = unsafe { impls::read_return_data(b.as_mut_ptr(), 0, i) };
    }
    (addr, b, l)
}

pub fn create1_slice_res<const REVERT_CAP: usize>(
    code: &[u8],
    endowment: U,
) -> Result<Address, ([u8; REVERT_CAP], usize)> {
    let (addr, b, l) = create1_slice(code, endowment);
    if addr != [0u8; 20] {
        Ok(addr)
    } else {
        Err((b, l))
    }
}

#[cfg(feature = "alloc")]
pub fn create1_vec(code: &[u8], endowment: U) -> (Address, Option<Vec<u8>>) {
    let (addr, rd) = create1_partial(code, endowment);
    if addr != [0u8; 20] {
        (addr, None)
    } else {
        let mut b = Vec::with_capacity(rd);
        unsafe { impls::read_return_data(b.as_mut_ptr(), 0, rd) };
        unsafe { b.set_len(rd) }
        (addr, Some(b))
    }
}

pub fn create2_partial(code: &[u8], endowment: U, salt: U) -> (Address, usize) {
    let mut addr = [0u8; 20];
    let mut revert_len = 0;
    unsafe {
        impls::create2(
            code.as_ptr(),
            code.len(),
            endowment.as_ptr(),
            salt.as_ptr(),
            addr.as_mut_ptr(),
            &mut revert_len as *mut usize,
        )
    }
    (addr, revert_len)
}

pub fn create2_slice<const REVERT_CAP: usize>(
    code: &[u8],
    endowment: U,
    salt: U,
) -> (Address, [u8; REVERT_CAP], usize) {
    let (addr, rd) = create2_partial(code, endowment, salt);
    let mut b = [0u8; REVERT_CAP];
    if addr == [0u8; 20] {
        assert!(REVERT_CAP >= rd, "create2 not enough space");
        unsafe { impls::read_return_data(b.as_mut_ptr(), 0, rd) };
    }
    (addr, b, rd)
}

pub fn create2_slice_res<const REVERT_CAP: usize>(
    code: &[u8],
    endowment: U,
    salt: U,
) -> Result<Address, ([u8; REVERT_CAP], usize)> {
    let (addr, b, l) = create2_slice(code, endowment, salt);
    if addr != [0u8; 20] {
        Ok(addr)
    } else {
        Err((b, l))
    }
}

pub fn create2_post_unit(code: &[u8], endowment: U, salt_digest: U) -> Option<Address> {
    let (addr, _) = create2_partial(code, endowment, salt_digest);
    if addr == [0u8; 20] {
        return None;
    }
    Some(addr)
}

pub fn create2_pre_unit(code: &[u8], endowment: U, salt_pre: &[u8]) -> Option<Address> {
    create2_post_unit(code, endowment, keccak256(salt_pre))
}

#[cfg(feature = "alloc")]
pub fn create2_post_vec(code: &[u8], endowment: U, salt: U) -> (Address, Option<Vec<u8>>) {
    let (addr, rd) = create2_partial(code, endowment, salt);
    if addr == [0u8; 20] {
        let mut b = Vec::with_capacity(rd);
        unsafe { impls::read_return_data(b.as_mut_ptr(), 0, rd) };
        unsafe {
            b.set_len(rd);
        }
        return (addr, Some(b));
    }
    (addr, None)
}

pub fn create2_slice_pre_keccak256<const REVERT_CAP: usize>(
    code: &[u8],
    endowment: U,
    salt_pre: &[u8],
) -> (Address, [u8; REVERT_CAP], usize) {
    create2_slice::<REVERT_CAP>(code, endowment, const_keccak256(salt_pre))
}

#[cfg(feature = "alloc")]
pub fn create2_vec_pre_keccak256(
    code: &[u8],
    endowment: U,
    salt_pre: &[u8],
) -> (Address, Option<Vec<u8>>) {
    create2_post_vec(code, endowment, keccak256(salt_pre))
}

pub fn const_estimate_addr_pre(factory: Address, initcode_pre: &[u8], salt_pre: &[u8]) -> Address {
    const_estimate_addr_post(
        factory,
        const_keccak256(initcode_pre),
        const_keccak256(salt_pre),
    )
}

/// Estimate the address of the create2 deployment.
pub fn const_estimate_addr_post(factory: Address, initcode_digest: U, salt_digest: U) -> Address {
    let b: [u8; 1 + 20 + 32 * 2] =
        concat_arrays!([0xff], factory, salt_digest.0, initcode_digest.0);
    const_keccak256(&b).into()
}

pub fn estimate_addr(factory: Address, initcode: U, salt: U) -> Address {
    let b: [u8; 1 + 20 + 32 * 2] = concat_arrays!([0xff], factory, salt.0, initcode.0);
    keccak256(&b).into()
}

/// Estimate the address of the create2 deployment.
pub fn estimate_addr_pre(factory: Address, initcode_pre: &[u8], salt_pre: &[u8]) -> Address {
    estimate_addr(factory, keccak256(initcode_pre), keccak256(salt_pre))
}
