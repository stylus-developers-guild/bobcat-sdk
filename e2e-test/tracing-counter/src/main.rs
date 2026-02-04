#![no_main]
#![no_std]

use bobcat_sdk::prelude::*;

#[global_allocator]
static ALLOC: bobcat_alloc = bobcat_alloc::INIT;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn user_entrypoint(_: usize) -> usize {
    bump();
    write_result_word(&transient_load(&U(SLOT_TRACING_COUNTER)));
    0
}
