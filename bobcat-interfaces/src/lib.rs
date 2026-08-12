#![cfg_attr(not(test), no_std)]

mod sels;

pub mod eip1967;
pub mod eip20;
pub mod eip2612;

pub mod aave_v3;
pub mod abracadabra;
pub mod aevo;
pub mod apx_bridge;
pub mod arrakis_modular;
pub mod basin;
pub mod beefy;
pub mod chamber_vaults;
pub mod boros;
pub mod buidl;
pub mod camelotv3_swap_router;
pub mod compound_v3;
pub mod concrete;
pub mod contango_v2;
pub mod cian_yield_layer;
pub mod convex;
pub mod csigma;
pub mod curve;
pub mod d2_finance;
pub mod dexalot;
pub mod enzyme;
pub mod deltaprime;
pub mod defi_saver;
pub mod derive_v2;
pub mod dolomite;
pub mod estate_protocol;
pub mod evedex;
pub mod fluid_dex;
pub mod fluid_lending;
pub mod hegic;
pub mod gains_network;
pub mod gmx_v2;
pub mod hibachi_bridge;
pub mod hyperliquid_bridge;
pub mod lagoon;
pub mod midas_rwa;
pub mod morpho;
pub mod native_credit;
pub mod near_intents;
pub mod omniyield;
pub mod ondo_yield;
pub mod orderly_bridge;
pub mod mux_perps;
pub mod ostium;
pub mod pancakeswap_v3;
pub mod pendle;
pub mod penpie;
pub mod radpie;
pub mod railgun;
pub mod renzo;
pub mod revert_lend;
pub mod spark_savings;
pub mod spiko;
pub mod steakhouse;
pub mod stargate_v2;
pub mod sushiswap_v2;
pub mod stobox;
pub mod t3tris;
pub mod theo_thbill;
pub mod toros;
pub mod txflow_bridge;
pub mod uniswap_v2;
pub mod uniswap_v3;
pub mod uniswap_v4;
pub mod usd_ai;
pub mod yield_yak;
pub mod zerobase;
pub mod veda;

pub mod chainlink_price_feed;
pub mod chainlink_vrf;

pub mod ninelives;

pub mod superposition;

pub use bobcat_cd;
