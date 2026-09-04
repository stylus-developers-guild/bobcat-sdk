#![cfg_attr(not(test), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod arbitrum;
pub mod ethereum;
pub mod markovgeist;
pub mod superposition;

pub use bobcat_maths::U;