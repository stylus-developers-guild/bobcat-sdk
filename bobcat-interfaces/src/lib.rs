#![cfg_attr(not(test), no_std)]

mod sels;

pub mod eip1967;
pub mod eip20;
pub mod eip2612;

pub mod aave_v3;
pub mod camelotv3_swap_router;
pub mod compound_v3;

pub mod chainlink_price_feed;
pub mod chainlink_vrf;

pub mod ninelives;

pub mod superposition;

pub use bobcat_cd;
