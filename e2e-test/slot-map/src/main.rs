#![no_main]
#![no_std]

use bobcat_sdk::prelude::*;

#[global_allocator]
static ALLOC: bobcat_alloc = bobcat_alloc::INIT;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn user_entrypoint(args_len: usize) -> usize {
    let args = &read_args_safe!(args_len, { (32 * 2) + 4 });
    let (x, y) = read_words!(&args[4..], 2);
    write_result_word(&slot_map(x, y));
    0
}
