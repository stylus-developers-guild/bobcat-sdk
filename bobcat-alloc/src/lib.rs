#![no_std]

#[cfg(all(target_family = "wasm", target_os = "unknown"))]
pub use mini_alloc::MiniAlloc as Alloc;

#[macro_export]
macro_rules! bobcat_allocator {
    () => {
        #[cfg(any(
            all(target_family = "wasm", target_os = "unknown"),
            all(target_arch = "riscv32", target_os = "none")
        ))]
        #[global_allocator]
        static ALLOC: $crate::Alloc = $crate::Alloc;
        extern crate alloc;
    };
}
