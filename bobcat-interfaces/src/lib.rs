#![cfg_attr(not(test), no_std)]

mod sels;

pub mod eip1967;
pub mod eip20;
pub mod eip2612;

pub mod aave_v3;
pub mod camelotv3_swap_router;
pub mod compound_v3;
pub mod gmx_v2;
pub mod morpho;
pub mod pendle;
pub mod ramses_v3;
pub mod robinhood_stock_tokens;
pub mod uniswap_v2;
pub mod uniswap_v3;
pub mod uniswap_v4;

pub mod chainlink_price_feed;
pub mod chainlink_vrf;

pub mod ninelives;

pub mod stargate;
pub mod superposition;

pub use bobcat_cd;
