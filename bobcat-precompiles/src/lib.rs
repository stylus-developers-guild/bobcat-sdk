#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod ethereum;
pub mod markovgeist;
pub mod superposition;

pub use bobcat_maths::U;
