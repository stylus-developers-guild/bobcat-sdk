// console-log-txt: Print the results of adding two numbers together
// before returning.

#![no_std]
#![no_main]

use bobcat_sdk::{alloc::bobcat_allocator, cd::*, console::console, entry::*};

bobcat_allocator!();

#[unsafe(no_mangle)]
pub unsafe extern "C" fn user_entrypoint(args_len: usize) -> usize {
    let args = &read_args_safe!(args_len, { 32 * 2 + 4 });
    let (x, y) = read_words!(&args[4..], 2);
    let z = x + y;
    console!(x, y, z);
    write_result_word(&z);
    0
}
