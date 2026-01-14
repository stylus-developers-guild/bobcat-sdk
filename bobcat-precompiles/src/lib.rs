#![no_std]

#[cfg(feature = "ed25519-dalek")]
mod ed25519;

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod ethereum;
pub mod superposition;
pub mod markovgeist;
