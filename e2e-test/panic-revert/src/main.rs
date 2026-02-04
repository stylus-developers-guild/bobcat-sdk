#![no_std]
#![no_main]

use bobcat_sdk::panic::panic_on_err_overflow;

#[global_allocator]
static ALLOC: bobcat_alloc = bobcat_alloc::INIT;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn user_entrypoint(_: usize) -> usize {
    panic_on_err_overflow!(None, "Hello!");
    0
}
