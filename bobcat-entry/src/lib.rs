#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

pub use bobcat_maths::U;

type Address = [u8; 20];

pub use bobcat_cd::read_words;

pub use bobcat_host as host;

pub fn balance(addr: Address) -> U {
    let mut out = U::ZERO;
    unsafe { host::account_balance(addr.as_ptr(), out.as_mut_ptr()) }
    out
}

#[unsafe(no_mangle)]
#[cfg(all(
    target_family = "wasm",
    target_os = "unknown",
    not(feature = "dont-define-symbols")
))]
pub unsafe fn mark_used() {
    unsafe { host::pay_for_memory_grow(0) }
    panic!();
}

pub fn write_result_slice(s: &[u8]) {
    unsafe { host::write_result(s.as_ptr(), s.len()) }
}

pub fn write_result_word(s: &U) {
    write_result_slice(&s.0)
}

pub fn write_result_bool(v: bool) {
    write_result_slice(&U::from(v).0)
}

pub fn return_data_size() -> usize {
    unsafe { host::return_data_size() }
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
        if rc { 0 } else { 1 }
    }};
}

pub fn read_args<const CAP: usize>(len: usize) -> ([u8; CAP], usize) {
    assert!(CAP >= len, "cap not enough");
    let mut b = [0u8; CAP];
    unsafe { host::read_args(b.as_mut_ptr()) };
    (b, len)
}

#[cfg(all(target_arch = "riscv32", target_os = "none"))]
pub fn args_len() -> usize {
    unsafe { host::args_len() }
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
        host::read_args(b.as_mut_ptr());
        b.set_len(len);
    };
    b
}

pub fn msg_sender() -> Address {
    let mut b = [0u8; 20];
    unsafe { host::msg_sender(b.as_mut_ptr()) }
    b
}

pub fn contract_address() -> Address {
    let mut b = [0u8; 20];
    unsafe { host::contract_address(b.as_mut_ptr()) }
    b
}

pub fn msg_value() -> U {
    let mut b = [0u8; 32];
    unsafe { host::msg_value(b.as_mut_ptr()) }
    U(b)
}

pub fn code_size(addr: Address) -> usize {
    unsafe { host::account_code_size(addr.as_ptr()) }
}

pub fn code_slice<const CAP: usize>(
    addr: Address,
    size: usize,
    offset: usize,
) -> ([u8; CAP], usize) {
    let mut b = [0u8; CAP];
    assert!(CAP >= size, "not enough size: {size}, capacity: {CAP}");
    let rd = unsafe { host::account_code(addr.as_ptr(), offset, size, b.as_mut_ptr()) };
    (b, rd)
}

#[cfg(feature = "alloc")]
pub fn code_vec_size(addr: Address, offset: usize, size: usize) -> Vec<u8> {
    let mut b = Vec::with_capacity(size);
    let rd = unsafe { host::account_code(addr.as_ptr(), offset, size, b.as_mut_ptr()) };
    unsafe { b.set_len(rd) };
    b
}

#[cfg(feature = "alloc")]
pub fn code_vec(addr: Address, offset: usize) -> Vec<u8> {
    code_vec_size(addr, offset, code_size(addr))
}

pub fn code_hash(addr: Address) -> U {
    let mut b = U::ZERO;
    unsafe { host::account_codehash(addr.as_ptr(), b.as_mut_ptr()) };
    b
}

pub fn chain_id() -> u64 {
    unsafe { host::chainid() }
}

pub fn block_timestamp() -> u64 {
    unsafe { host::block_timestamp() }
}

pub fn block_basefee() -> U {
    let mut out = U::ZERO;
    unsafe { host::block_basefee(out.as_mut_ptr()) }
    out
}

pub fn evm_gas_left() -> u64 {
    unsafe { host::evm_gas_left() }
}

pub fn evm_ink_left() -> u64 {
    unsafe { host::evm_ink_left() }
}

pub unsafe fn exit_early(code: usize) -> ! {
    unsafe { host::exit_early(code as i32) }
}

pub unsafe fn storage_flush_cache(clear: bool) {
    unsafe { host::storage_flush_cache(clear) }
}
