#![no_main]
#![no_std]

use bobcat_sdk::prelude::*;

bobcat_allocator!();

#[unsafe(no_mangle)]
pub unsafe extern "C" fn user_entrypoint(l: usize) -> usize {
    let (args, _) = read_args::<1024>(l);
    write_result_slice(&args);
    0
}
