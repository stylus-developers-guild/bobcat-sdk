#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

pub use bobcat_maths::U;

type Address = [u8; 20];

pub use bobcat_cd::read_words;

#[cfg(all(target_family = "wasm", target_os = "unknown"))]
mod impls {
    #[link(wasm_import_module = "vm_hooks")]
    unsafe extern "C" {
        pub(crate) fn account_balance(addr: *const u8, dest: *mut u8);
        #[allow(unused)]
        pub(crate) fn pay_for_memory_grow(pages: u16);
        pub(crate) fn write_result(d: *const u8, l: usize);
        pub(crate) fn return_data_size() -> usize;
        pub(crate) fn read_args(out: *mut u8);
        pub(crate) fn msg_sender(addr: *mut u8);
        pub(crate) fn contract_address(addr: *mut u8);
        pub(crate) fn msg_value(value: *mut u8);
        pub fn chainid() -> u64;
        pub(crate) fn account_code_size(address: *const u8) -> usize;
        pub(crate) fn account_code(
            address: *const u8,
            offset: usize,
            size: usize,
            dest: *mut u8,
        ) -> usize;
        pub(crate) fn account_codehash(address: *const u8, dest: *mut u8);
        pub(crate) fn block_timestamp() -> u64;
        pub(crate) fn block_basefee(out: *mut u8);
        pub(crate) fn evm_gas_left() -> u64;
        pub(crate) fn evm_ink_left() -> u64;
    }
}

#[cfg(all(
    not(all(target_family = "wasm", target_os = "unknown")),
    feature = "std"
))]
pub mod entry_host {
    use super::{Address, U};

    use core::{ptr::copy_nonoverlapping, slice::from_raw_parts};

    use std::{cell::RefCell, cmp::min, collections::HashMap};

    use bobcat_storage::keccak256;

    thread_local! {
        static ACCOUNT_BALANCE: RefCell<HashMap<Address, U>> = RefCell::default();
        static ARGS: RefCell<Vec<u8>> = RefCell::default();
        static MSG_SENDER: RefCell<[u8; 20]> = RefCell::default();
        static CONTRACT_ADDRESS: RefCell<Address> = RefCell::default();
        static MSG_VALUE: RefCell<U> = RefCell::default();
        static CHAIN_ID: RefCell<u64> = RefCell::default();
        static BLOCK_TIMESTAMP: RefCell<u64> = RefCell::default();
        static ACCOUNT_CODE: RefCell<HashMap<Address, Vec<u8>>> = RefCell::default();
    }

    const EMPTY_HASH: U = U([
        0xc5, 0xd2, 0x46, 0x01, 0x86, 0xf7, 0x23, 0x3c, 0x92, 0x7e, 0x7d, 0xb2, 0xdc, 0xc7, 0x03,
        0xc0, 0xe5, 0x00, 0xb6, 0x53, 0xca, 0x82, 0x27, 0x3b, 0x7b, 0xfa, 0xd8, 0x04, 0x5d, 0x85,
        0xa4, 0x70,
    ]);

    pub(crate) unsafe fn account_balance(addr_: *const u8, out: *mut u8) {
        ACCOUNT_BALANCE.with(|s| {
            let mut addr = [0u8; 20];
            unsafe {
                copy_nonoverlapping(addr_, addr.as_mut_ptr(), 32);
            }
            let h = s.borrow();
            let amt = h.get(&addr).unwrap_or(&U::ZERO);
            unsafe {
                copy_nonoverlapping(amt.as_ptr(), out, 32);
            }
        })
    }

    #[allow(unused)]
    pub(crate) unsafe fn pay_for_memory_grow(_: u16) {}

    pub(crate) unsafe fn write_result(d: *const u8, l: usize) {
        println!("{}", const_hex::encode(unsafe { from_raw_parts(d, l) }));
    }

    pub(crate) fn return_data_size() -> usize {
        0
    }

    pub fn set_args(x: Vec<u8>) {
        ARGS.with(|s| *s.borrow_mut() = x)
    }

    pub fn args_len() -> usize {
        ARGS.with(|s| s.borrow().len())
    }

    pub(crate) unsafe fn read_args(out: *mut u8) {
        ARGS.with(|s| {
            let b = s.borrow();
            unsafe {
                copy_nonoverlapping(b.as_ptr(), out, b.len());
            }
        })
    }

    pub fn set_msg_sender(x: Address) {
        MSG_SENDER.with(|s| *s.borrow_mut() = x)
    }

    pub(crate) unsafe fn msg_sender(out: *mut u8) {
        MSG_SENDER.with(|s| {
            let b = s.borrow();
            unsafe {
                copy_nonoverlapping(b.as_ptr(), out, 20);
            }
        })
    }

    pub fn set_contract_address(x: Address) {
        CONTRACT_ADDRESS.with(|s| {
            *s.borrow_mut() = x;
        })
    }

    pub fn set_account_code(x: Address, code: Vec<u8>) {
        ACCOUNT_CODE.with(|s| s.borrow_mut().insert(x, code));
    }

    pub(crate) unsafe fn contract_address(out: *mut u8) {
        CONTRACT_ADDRESS.with(|s| {
            let b = s.borrow();
            unsafe {
                copy_nonoverlapping(b.as_ptr(), out, 20);
            }
        })
    }

    pub(crate) unsafe fn msg_value(out: *mut u8) {
        MSG_VALUE.with(|s| {
            let b = s.borrow();
            unsafe {
                copy_nonoverlapping(b.as_ptr(), out, 32);
            }
        })
    }

    pub(crate) unsafe fn chainid() -> u64 {
        CHAIN_ID.with(|s| s.borrow().clone())
    }

    pub(crate) unsafe fn account_code_size(addr_: *const u8) -> usize {
        ACCOUNT_CODE.with(|s| {
            let mut addr = [0u8; 20];
            unsafe {
                copy_nonoverlapping(addr_, addr.as_mut_ptr(), 20);
            }
            s.borrow().get(&addr).map(|s| s.len()).unwrap_or(0)
        })
    }

    pub(crate) unsafe fn account_code(
        addr_: *const u8,
        offset: usize,
        size: usize,
        out: *mut u8,
    ) -> usize {
        ACCOUNT_CODE.with(|s| {
            let mut addr = [0u8; 20];
            unsafe {
                copy_nonoverlapping(addr_, addr.as_mut_ptr(), 20);
            }
            let b = s.borrow();
            match b.get(&addr) {
                Some(b) => {
                    if offset >= b.len() {
                        return 0;
                    }
                    let src = &b[offset..];
                    let len = min(size, src.len());
                    unsafe {
                        copy_nonoverlapping(src.as_ptr(), out, len);
                    }
                    len
                }
                None => 0,
            }
        })
    }

    pub unsafe fn account_codehash(addr_: *const u8, out: *mut u8) {
        ACCOUNT_CODE.with(|s| {
            let mut addr = [0u8; 20];
            unsafe {
                copy_nonoverlapping(addr_, addr.as_mut_ptr(), 20);
            }
            let b = s.borrow();
            let h = match b.get(&addr) {
                None => EMPTY_HASH,
                Some(b) if b.len() == 0 => EMPTY_HASH,
                Some(b) => keccak256(&b),
            };
            unsafe {
                copy_nonoverlapping(h.as_ptr(), out, 32);
            }
        })
    }

    pub fn set_block_timestamp(n: u64) {
        BLOCK_TIMESTAMP.with(|s| *s.borrow_mut() = n)
    }

    pub(crate) unsafe fn block_timestamp() -> u64 {
        BLOCK_TIMESTAMP.with(|s| *s.borrow())
    }

    pub(crate) unsafe fn block_basefee(out: *mut u8) {}

    pub(crate) unsafe fn evm_gas_left() -> u64 {
        0
    }

    pub(crate) fn evm_ink_left() -> u64 {
        0
    }
}

#[cfg(all(
    not(all(target_family = "wasm", target_os = "unknown")),
    feature = "std"
))]
pub use entry_host as impls;

#[cfg(all(
    not(all(target_family = "wasm", target_os = "unknown")),
    not(feature = "std")
))]
mod impls {
    pub(crate) fn account_balance(_: *const u8, _: *mut u8) {}

    #[allow(unused)]
    pub(crate) unsafe fn pay_for_memory_grow(_: u16) {}

    pub(crate) unsafe fn write_result(_: *const u8, _: usize) {}

    pub(crate) unsafe fn return_data_size() -> usize { 0 }

    pub(crate) unsafe fn read_args(_out: *mut u8) {}

    pub(crate) unsafe fn msg_sender(_: *mut u8) {}

    pub(crate) unsafe fn contract_address(_: *mut u8) {}

    pub(crate) unsafe fn msg_value(_: *mut u8) {}

    pub(crate) unsafe fn chainid() -> u64 {
        0
    }

    pub unsafe fn account_code_size(_: *const u8) -> usize {
        0
    }

    pub(crate) unsafe fn account_code(_: *const u8, _: usize, _: usize, _: *mut u8) -> usize {
        0
    }

    pub(crate) unsafe fn account_codehash(_: *const u8, _: *mut u8) {}

    pub(crate) unsafe fn block_timestamp() -> u64 {
        0
    }

    pub(crate) unsafe fn block_basefee(out: *mut u8) {}

    pub(crate) unsafe fn evm_gas_left() -> u64 {
        0
    }
    pub(crate) fn evm_ink_left() -> u64 {
        0
    }
}

pub fn balance(addr: Address) -> U {
    let mut out = U::ZERO;
    unsafe { impls::account_balance(addr.as_ptr(), out.as_mut_ptr()) }
    out
}

#[unsafe(no_mangle)]
#[cfg(all(
    target_family = "wasm",
    target_os = "unknown",
    not(feature = "dont-define-symbols")
))]
pub unsafe fn mark_used() {
    unsafe { impls::pay_for_memory_grow(0) }
    panic!();
}

pub fn write_result_slice(s: &[u8]) {
    unsafe { impls::write_result(s.as_ptr(), s.len()) }
}

pub fn write_result_word(s: &U) {
    write_result_slice(&s.0)
}

pub fn write_result_bool(v: bool) {
    write_result_slice(&U::from(v).0)
}

pub fn return_data_size() -> usize {
    unsafe { impls::return_data_size() }
}

pub use bobcat_cd::leftpad_addr;

/// Like write_result_exit_call, except it only reverts with the
/// returndata if the underlying call reverted. If it doesn't, then it
/// just returns the slice.
#[macro_export]
macro_rules! revert_if_bad_call_vec {
    ($e:expr) => {{
        let (rc, rd) = $e;
        if !rc {
            $crate::write_result_slice(&rd);
            return 1;
        }
        rd
    }};
}

/// Reverts if the underlying call failed, using the vector that was
/// returned as the third argument as slice.
#[macro_export]
macro_rules! revert_if_bad_call_unit_vec {
    ($e:expr) => {{
        let (rc, revertdata) = $e;
        match (rc, revertdata) {
            (true, _) => (),
            (false, Some(v)) => {
                $crate::write_result_slice(&v);
                return 1;
            }
            (false, _) => return 1,
        }
    }};
}

/// Reverts with a message if the revertdata is Some, and if the rc is false.
#[macro_export]
macro_rules! revert_if_bad_call_slice_vec {
    ($e:expr) => {{
        let (rc, returndata, revertdata) = $e;
        match (rc, revertdata) {
            (true, _) => returndata,
            (false, Some(v)) => {
                $crate::write_result_slice(&v);
                return 1;
            }
            (false, _) => return 1,
        }
    }};
}

#[macro_export]
macro_rules! write_result_exit_res {
    ($ident:expr) => {{
        match $ident {
            Ok(v) => {
                $crate::write_result_slice(&v);
                0
            }
            Err(v) => {
                $crate::write_result_slice(&v);
                1
            }
        }
    }};
}

#[macro_export]
macro_rules! write_result_exit_create {
    ($ident:expr) => {{
        let (addr, b, i) = $ident;
        if addr != [0u8; 20] {
            $crate::write_result_slice(&leftpad_addr(addr));
            0
        } else {
            $crate::write_result_slice(&b[..i]);
            1
        }
    }};
}

#[macro_export]
macro_rules! write_result_exit_call {
    ($ident:expr) => {{
        let (rc, l, v) = $ident;
        $crate::write_result_slice(&v[..l]);
        if rc {
            0
        } else {
            1
        }
    }};
}

pub fn read_args<const CAP: usize>(len: usize) -> ([u8; CAP], usize) {
    assert!(CAP >= len, "cap not enough");
    let mut b = [0u8; CAP];
    unsafe { impls::read_args(b.as_mut_ptr()) };
    (b, len)
}

#[macro_export]
macro_rules! read_args_safe {
    ($len:expr, $max_len:expr) => {{
        assert!($max_len >= $len, "{} < {}", $max_len, $len);
        $crate::read_args::<$max_len>($len).0
    }};
}

#[cfg(feature = "alloc")]
pub fn read_args_vec(len: usize) -> Vec<u8> {
    let mut b = Vec::with_capacity(len);
    unsafe {
        impls::read_args(b.as_mut_ptr());
        b.set_len(len);
    };
    b
}

pub fn msg_sender() -> Address {
    let mut b = [0u8; 20];
    unsafe { impls::msg_sender(b.as_mut_ptr()) }
    b
}

pub fn contract_address() -> Address {
    let mut b = [0u8; 20];
    unsafe { impls::contract_address(b.as_mut_ptr()) }
    b
}

pub fn msg_value() -> U {
    let mut b = [0u8; 32];
    unsafe { impls::msg_value(b.as_mut_ptr()) }
    U(b)
}

pub fn code_size(addr: Address) -> usize {
    unsafe { impls::account_code_size(addr.as_ptr()) }
}

pub fn code_slice<const CAP: usize>(
    addr: Address,
    size: usize,
    offset: usize,
) -> ([u8; CAP], usize) {
    let mut b = [0u8; CAP];
    assert!(CAP >= size, "not enough size: {size}, capacity: {CAP}");
    let rd = unsafe { impls::account_code(addr.as_ptr(), offset, size, b.as_mut_ptr()) };
    (b, rd)
}

#[cfg(feature = "alloc")]
pub fn code_vec_size(addr: Address, offset: usize, size: usize) -> Vec<u8> {
    let mut b = Vec::with_capacity(size);
    let rd = unsafe { impls::account_code(addr.as_ptr(), offset, size, b.as_mut_ptr()) };
    unsafe { b.set_len(rd) };
    b
}

#[cfg(feature = "alloc")]
pub fn code_vec(addr: Address, offset: usize) -> Vec<u8> {
    code_vec_size(addr, offset, code_size(addr))
}

pub fn code_hash(addr: Address) -> U {
    let mut b = U::ZERO;
    unsafe { impls::account_codehash(addr.as_ptr(), b.as_mut_ptr()) };
    b
}

pub fn chain_id() -> u64 {
    unsafe { impls::chainid() }
}

pub fn block_timestamp() -> u64 {
    unsafe { impls::block_timestamp() }
}

pub fn block_basefee() -> U {
    let mut out = U::ZERO;
    unsafe { impls::block_basefee(out.as_mut_ptr()) }
    out
}

pub fn evm_gas_left() -> u64 {
    unsafe { impls::evm_gas_left() }
}

pub fn evm_ink_left() -> u64 {
    unsafe { impls::evm_ink_left() }
}
