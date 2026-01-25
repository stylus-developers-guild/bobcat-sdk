#![cfg_attr(any(target_arch = "wasm32", target_arch = "riscv32"), no_std, no_main)]

#[unsafe(no_mangle)]
pub unsafe extern "C" fn user_entrypoint(len: usize) -> usize {
    libbozo::user_entrypoint(len)
}

#[allow(unused)]
fn main() {}
