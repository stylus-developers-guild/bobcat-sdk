#![no_main]
#![no_std]

use bobcat_sdk::prelude::*;

bobcat_allocator!();

const SEL_NUMBER: [u8; 4] = const_keccak_sel(b"number()");
const SEL_SET_NUMBER: [u8; 4] = const_keccak_sel(b"setNumber(uint256)");
const SEL_MUL_NUMBER: [u8; 4] = const_keccak_sel(b"mulNumber(uint256)");
const SEL_ADD_NUMBER: [u8; 4] = const_keccak_sel(b"addNumber(uint256)");
const SEL_INCREMENT: [u8; 4] = const_keccak_sel(b"increment()");
const SEL_ADD_FROM_MSG_VALUE: [u8; 4] = const_keccak_sel(b"addFromMsgValue()");

#[link(wasm_import_module = "vm_hooks")]
unsafe extern "C" {
     fn msg_reentrant() -> bool;
 }

#[unsafe(no_mangle)]
pub unsafe extern "C" fn user_entrypoint(args_len: usize) -> usize {
   assert!(! unsafe { msg_reentrant() });
    let args = read_args_safe!(args_len, { 32 + 4 });
    let sel: [u8; 4] = args[..4].try_into().unwrap();
    let w = read_words!(&args[4..], 1);
    flush_guard(|| match sel {
        SEL_NUMBER => write_result_word(&storage_load(&U::ZERO)),
        SEL_SET_NUMBER => storage_store(&U::ZERO, w),
        SEL_MUL_NUMBER => storage_wrapping_mul(&U::ZERO, w),
        SEL_ADD_NUMBER => storage_wrapping_add(&U::ZERO, w),
        SEL_INCREMENT => storage_wrapping_add(&U::ONE, w),
        SEL_ADD_FROM_MSG_VALUE => storage_wrapping_add(&msg_value(), w),
        _ => unimplemented!(),
    });
    0
}
