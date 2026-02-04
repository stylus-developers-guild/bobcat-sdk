#![no_main]
#![no_std]

use bobcat_sdk::{
    alloc::Alloc,
    cd::{const_keccak_sel, read_words},
    entry::*,
    maths::U,
    storage::*,
};

#[global_allocator]
pub static INIT: Alloc = Alloc;

pub fn get_number() -> U {
    storage_load(&U::ZERO)
}

pub fn set_number(x: &U) {
    storage_store(&U::ZERO, x)
}

pub fn mul_number(x: &U) {
    storage_wrapping_mul(&U::ZERO, x)
}

pub fn add_number(x: &U) {
    storage_wrapping_add(&U::ZERO, x)
}

#[link(wasm_import_module = "vm_hooks")]
unsafe extern "C" {
    fn msg_reentrant() -> bool;
}

pub const SEL_NUMBER: [u8; 4] = const_keccak_sel(b"number()");
pub const SEL_SET_NUMBER: [u8; 4] = const_keccak_sel(b"setNumber(uint256)");
pub const SEL_MUL_NUMBER: [u8; 4] = const_keccak_sel(b"mulNumber(uint256)");
pub const SEL_ADD_NUMBER: [u8; 4] = const_keccak_sel(b"addNumber(uint256)");
pub const SEL_INCREMENT: [u8; 4] = const_keccak_sel(b"increment()");
pub const SEL_ADD_FROM_MSG_VALUE: [u8; 4] = const_keccak_sel(b"addFromMsgValue()");

#[unsafe(no_mangle)]
pub unsafe extern "C" fn user_entrypoint(args_len: usize) -> usize {
    assert!(!unsafe { msg_reentrant() });
    let args = read_args_safe!(args_len, { 32 + 4 });
    let w = read_words!(&args[4..], 1);
    flush_guard(|| match args[..4].try_into().unwrap() {
        SEL_NUMBER => write_result_slice(get_number().as_slice()),
        SEL_SET_NUMBER => set_number(w),
        SEL_MUL_NUMBER => mul_number(w),
        SEL_ADD_NUMBER => add_number(w),
        SEL_INCREMENT => add_number(&U::ONE),
        SEL_ADD_FROM_MSG_VALUE => add_number(&msg_value()),
        _ => unimplemented!(),
    });
    0
}
