#![no_main]
#![no_std]

use bobcat_sdk::prelude::*;

#[link(wasm_import_module = "vm_hooks")]
unsafe extern "C" {
    fn msg_reentrant() -> bool;
}

#[derive(Debug, Clone, EvmCdSerialise, EvmCdDeserialise)]
#[evm_entrypoint]
pub enum Entry {
    Number,
    SetNumber(U),
    MulNumber(U),
    AddNumber(U),
    Increment,
    AddFromMsgValue,
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn user_entrypoint(args_len: usize) -> usize {
    assert!(!unsafe { msg_reentrant() });
    flush_guard(|| match read_cd::<_>(args_len) {
        Entry::Number => write_result_word(&storage_load(&U::ZERO)),
        Entry::SetNumber(w) => storage_store(&U::ZERO, &w),
        Entry::MulNumber(w) => storage_wrapping_mul(&U::ZERO, &w),
        Entry::AddNumber(w) => storage_wrapping_add(&U::ZERO, &w),
        Entry::Increment => storage_wrapping_add(&U::ZERO, &U::ONE),
        Entry::AddFromMsgValue => storage_wrapping_add(&U::ZERO, &msg_value()),
    });
    0
}
