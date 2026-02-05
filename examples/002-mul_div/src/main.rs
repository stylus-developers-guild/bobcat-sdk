#![no_main]
#![no_std]

use bobcat_sdk::{cd::read_words, entry::*, maths::U, alloc::bobcat_allocator};

bobcat_allocator!();

#[unsafe(no_mangle)]
pub unsafe extern "C" fn user_entrypoint(args_len: usize) -> usize {
    let args = &read_args_safe!(args_len, { (32 * 2) + 4 })[4..];
    let (x, y) = read_words!(&args, 2);
    let x = <&U>::from(x);
    let y = <&U>::from(y);
    write_result_slice(&x.mul_div(&y, U::from(100u32)).unwrap().0.0);
    0
}
