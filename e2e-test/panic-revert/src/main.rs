#![no_std]
#![no_main]

use bobcat_sdk::{alloc::bobcat_allocator, panic::panic_on_err_overflow};

bobcat_allocator!();

#[unsafe(no_mangle)]
pub unsafe extern "C" fn user_entrypoint(_: usize) -> usize {
    panic_on_err_overflow!(None; "Hello!");
    0
}
