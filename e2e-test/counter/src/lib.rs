#![no_main]
#![no_std]

use bobcat_sdk::prelude::*;

bobcat_allocator!();

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
pub unsafe extern "C" fn user_entrypoint(len: usize) -> usize {
    flush_guard(|| match read_cd::<Entry>(len) {
        Entry::Number => write_result_word(&storage_load(&U::ZERO)),
        Entry::SetNumber(w) => storage_store(&U::ZERO, &w),
        Entry::MulNumber(w) => storage_wrapping_mul(&U::ZERO, &w),
        Entry::AddNumber(w) => storage_wrapping_add(&U::ZERO, &w),
        Entry::Increment => storage_wrapping_add(&U::ZERO, &U::ONE),
        Entry::AddFromMsgValue => storage_wrapping_add(&U::ZERO, &msg_value()),
    });
    0
}
