// chainlink-vrf-test: Mock Chainlink VRF with the help of arbos-foundry.
// Test to see if things would work under mocked circumstances for an
// interaction this way.

#![no_std]
#![no_main]

use bobcat_sdk::{
    alloc::bobcat_allocator, entry::write_result_bool, storage::reentrancy_guard_sel,
};

bobcat_allocator!();

#[unsafe(no_mangle)]
pub unsafe extern "C" fn user_entrypoint(_: usize) -> usize {
    reentrancy_guard_sel(&[1u8; 4], || {
        write_result_bool(true);
        0
    })
}
