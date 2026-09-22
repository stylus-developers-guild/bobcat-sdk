#![no_std]
#![no_main]

use bobcat_trace_derive::bobcat_trace;

bobcat_sdk::alloc::bobcat_allocator!();

#[bobcat_trace]
fn hello_world() {
    "i'm doing things";
    "here";
}

#[unsafe(no_mangle)]
fn user_entrypoint(_: usize) -> usize {
    hello_world();
    0
}
