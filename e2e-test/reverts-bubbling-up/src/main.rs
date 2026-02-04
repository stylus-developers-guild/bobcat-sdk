#![no_std]
#![no_main]

use bobcat_sdk::{
    call::safe_call_bool_err_vec,
    entry::{
        contract_address, msg_sender, read_args_safe, read_words, revert_if_bad_call_unit_vec,
    },
    interfaces::eip20::make_fn_transfer_from,
    maths::{u, U},
};

#[global_allocator]
static ALLOC: bobcat_alloc = bobcat_alloc::INIT;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn user_entrypoint(len: usize) -> usize {
    let args = &read_args_safe!(len, { 32 * 4 + 32 });
    let target = read_words!(&args[4..], 1);
    // Test that this should bubble up:
    revert_if_bad_call_unit_vec!(safe_call_bool_err_vec(
        target.into(),
        &make_fn_transfer_from(msg_sender(), contract_address(), &u!(123123)),
        &U::ZERO,
        u64::MAX,
    ));
    0
}
