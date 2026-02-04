// chainlink-vrf-test: Mock Chainlink VRF with the help of arbos-foundry.
// Test to see if things would work under mocked circumstances for an
// interaction this way.

#![no_std]
#![no_main]

#[global_allocator]
static ALLOC: bobcat_alloc = bobcat_alloc::INIT;

use bobcat_sdk::{storage::reentrancy_guard_sel, entry::write_result_bool};

#[unsafe(no_mangle)]
pub unsafe extern "C" fn user_entrypoint(_: usize) -> usize {
    reentrancy_guard_sel(&[1u8; 4], || {
        write_result_bool(true);
        0
    })
}
