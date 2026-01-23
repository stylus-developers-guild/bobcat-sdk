#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(all(target_arch = "riscv32", target_os = "none"))]
mod risc;

#[cfg(all(target_family = "wasm", target_os = "unknown"))]
mod wasm;

#[cfg(all(target_arch = "riscv32", target_os = "none"))]
pub use risc::*;

#[cfg(all(target_family = "wasm", target_os = "unknown"))]
pub use wasm::*;

#[cfg(all(
    not(feature = "std"),
    not(any(
        all(target_family = "wasm", target_os = "unknown"),
        all(target_arch = "riscv32", target_os = "none")
    ))
))]
mod sys_nostd;

#[cfg(all(
    not(feature = "std"),
    not(any(
        all(target_family = "wasm", target_os = "unknown"),
        all(target_arch = "riscv32", target_os = "none")
    ))
))]
pub use sys_nostd::*;

#[cfg(all(
    feature = "std",
    not(any(
        all(target_family = "wasm", target_os = "unknown"),
        all(target_arch = "riscv32", target_os = "none")
    ))
))]
mod sys_std;

#[cfg(all(
    feature = "std",
    not(any(
        all(target_family = "wasm", target_os = "unknown"),
        all(target_arch = "riscv32", target_os = "none")
    ))
))]
pub use sys_std::*;
