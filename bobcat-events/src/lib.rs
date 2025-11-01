#![no_std]

use bobcat_maths::U;

use array_concat::concat_arrays;

#[cfg(target_arch = "wasm32")]
mod impls {
    #[link(wasm_import_module = "vm_hooks")]
    unsafe extern "C" {
        pub(crate) fn emit_log(data: *const u8, len: usize, topics: usize);
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod impls {
    pub(crate) fn emit_log(_: *const u8, _: usize, _: usize) {}
}

pub fn emit_log_0_slice<const D: usize, const ALL: usize>(t0: &U, d: [u8; D]) {
    assert_eq!(ALL, D + 32, "not properly sized: {}", D + 32);
    let x: [u8; ALL] = concat_arrays!(t0.0, d);
    unsafe { impls::emit_log(x.as_ptr(), x.len(), 1); }
}

pub fn emit_log_1_slice<const D: usize, const ALL: usize>(t0: &U, t1: &U, d: [u8; D]) {
    assert_eq!(ALL, D + 64, "not properly sized: {}", D + 64);
    let x: [u8; ALL] = concat_arrays!(t0.0, t1.0, d);
    unsafe { impls::emit_log(x.as_ptr(), x.len(), 2); }
}

pub fn emit_log_2_slice<const D: usize, const ALL: usize>(t0: &U, t1: &U, t2: &U, d: [u8; D]) {
    assert_eq!(ALL, D + 96, "not properly sized: {}", D + 96);
    let x: [u8; ALL] = concat_arrays!(t0.0, t1.0, t2.0, d);
    unsafe { impls::emit_log(x.as_ptr(), x.len(), 3); }
}

pub fn emit_log_3_slice<const D: usize, const ALL: usize>(t0: &U, t1: &U, t2: &U, t3: &U, d: [u8; D]) {
    assert_eq!(ALL, D + 128, "not properly sized: {}", D + 128);
    let x: [u8; ALL] = concat_arrays!(t0.0, t1.0, t2.0, t3.0, d);
    unsafe { impls::emit_log(x.as_ptr(), x.len(), 4); }
}

pub fn emit_log_4_slice<const D: usize, const ALL: usize>(t0: &U, t1: &U, t2: &U, t3: &U, t4: &U, d: [u8; D]) {
    assert_eq!(ALL, D + 160, "not properly sized: {}", D + 160);
    let x: [u8; ALL] = concat_arrays!(t0.0, t1.0, t2.0, t3.0, t4.0, d);
    unsafe { impls::emit_log(x.as_ptr(), x.len(), 5); }
}