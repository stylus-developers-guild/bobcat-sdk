//! Calldata builders for Dexalot Portfolio bridge core end-user flows.
//!
//! Dexalot is an omni-chain order-book DEX.  Rather than bridging tokens
//! between chains, Dexalot locks assets in a `PortfolioMain` contract on each
//! supported mainnet (Arbitrum, Avalanche, Base, …) and mirrors the user's
//! balance on the Dexalot L1 (subnet) where trading happens.  Cross-chain
//! messages are carried by bridge-agnostic `PortfolioBridgeMain` /
//! `PortfolioBridgeSub` aggregators (LayerZero, ICM, …).
//!
//! ## Architecture
//!
//! - **PortfolioMain** (mainnet / Arbitrum side) — user deposits native or
//!   ERC-20 tokens.  The tokens are locked in-situ and a deposit message is
//!   sent to the Dexalot L1.  Withdrawals are *not* user-initiated on the
//!   mainnet side: when a user requests a withdrawal on the Dexalot L1, the
//!   subnet sends a `WITHDRAW` XFER message back to `PortfolioMain`, which
//!   releases the locked funds to the user's wallet via `processXFerPayload`
//!   (callable only by the bridge).
//!
//! - **PortfolioSub** (Dexalot L1 subnet side) — users withdraw tokens to a
//!   destination mainnet chain.  `withdrawToken` sends a `WITHDRAW` XFER
//!   message through `PortfolioBridgeSub`; when the message reaches the
//!   destination `PortfolioMain`, the funds are released.  `withdrawNative`
//!   mints native ALOT from the `PortfolioMinter`.
//!
//! ## End-user functions exposed
//!
//! ### Deposit (PortfolioMain — Arbitrum / mainnet)
//!
//! - `depositNative(address _from, BridgeProvider _bridge)` — payable;
//!   deposits `msg.value` of the chain's native token.  `_from` must equal
//!   `msg.sender`.
//! - `depositToken(address _from, bytes32 _symbol, uint256 _quantity, BridgeProvider _bridge)` —
//!   payable (for bridge fee); deposits `_quantity` of the ERC-20 token
//!   identified by `_symbol`.  The caller must have ERC-20-approved
//!   `PortfolioMain` for `_quantity` beforehand.  `_from` must equal
//!   `msg.sender`.
//!
//! ### Withdraw (PortfolioSub — Dexalot L1 subnet)
//!
//! - `withdrawNative(address payable _to, uint256 _quantity)` — withdraws
//!   `_quantity` of native ALOT from the caller's subnet balance; the
//!   `PortfolioMinter` mints it to `_to`.  `_to` must equal `msg.sender`.
//! - `withdrawToken(address _to, bytes32 _symbol, uint256 _quantity, BridgeProvider _bridge, uint32 _dstChainListOrgChainId)` —
//!   withdraws `_quantity` of `_symbol` to the caller's address on the
//!   destination mainnet chain.  `_to` must equal `msg.sender`.
//! - `withdrawToken(address _from, bytes32 _to, bytes32 _symbol, uint256 _quantity, BridgeProvider _bridge, uint32 _dstChainListOrgChainId, bytes1 _options)` —
//!   extended variant: allows a `bytes32` destination address and an
//!   `_options` byte (e.g. auto-fill gas).  `_from` must equal `msg.sender`.
//!
//! ## Not exposed
//!
//! `depositTokenFromContract` (trusted-contract only),
//! `processXFerPayload` (bridge-role only), `collectBridgeFees` (admin only),
//! `sendXChainMessage` (bridge-user-role only), and all admin / pause /
//! configuration functions are intentionally omitted.
//!
//! ## ABI source
//!
//! Function signatures and selectors verified against the official Dexalot
//! contract source at
//! <https://github.com/Dexalot/contracts>:
//! `contracts/interfaces/IPortfolio.sol`,
//! `contracts/interfaces/IPortfolioMain.sol`,
//! `contracts/interfaces/IPortfolioSub.sol`,
//! `contracts/interfaces/IPortfolioBridge.sol`,
//! `contracts/PortfolioMain.sol`,
//! `contracts/PortfolioSub.sol`.
//!
//! All builders are `no_std` and allocation-free.  Every function has a fully
//! static ABI (no dynamic types) and returns a fixed-size array.

use array_concat::concat_arrays;
use bobcat_cd::{leftpad_addr, leftpad_u32, leftpad_u8};
use bobcat_maths::U;

use crate::selectors;

/// An EVM address.
pub type Address = [u8; 20];

/// A Dexalot token symbol (`bytes32`, left-aligned ASCII, zero-padded).
pub type Symbol = [u8; 32];

/// A `bytes32` destination address (for cross-chain withdrawals to chains
/// with non-EVM address formats).
pub type Bytes32Addr = [u8; 32];

/// Bridge provider enum used by Dexalot's `PortfolioBridge`.
///
/// Maps to `IPortfolioBridge.BridgeProvider`:
/// - `LZ` = 0 (LayerZero)
/// - `CELER` = 1 (Celer)
/// - `ICM` = 2 (Avalanche ICM)
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum BridgeProvider {
    /// LayerZero.
    LZ = 0,
    /// Celer.
    CELER = 1,
    /// Avalanche ICM.
    ICM = 2,
}

impl BridgeProvider {
    /// Return the `uint8` discriminant used in ABI encoding.
    pub const fn as_u8(self) -> u8 {
        self as u8
    }
}

selectors! {
    SEL_DEPOSIT_NATIVE = b"depositNative(address,uint8)",
    SEL_DEPOSIT_TOKEN = b"depositToken(address,bytes32,uint256,uint8)",
    SEL_WITHDRAW_NATIVE = b"withdrawNative(address,uint256)",
    SEL_WITHDRAW_TOKEN = b"withdrawToken(address,bytes32,uint256,uint8,uint32)",
    SEL_WITHDRAW_TOKEN_EXT = b"withdrawToken(address,bytes32,bytes32,uint256,uint8,uint32,bytes1)",
}

// ---------------------------------------------------------------------------
// depositNative — deposit native token (PortfolioMain, payable)
// ---------------------------------------------------------------------------

/// Encode a Dexalot `depositNative(address _from, BridgeProvider _bridge)`
/// call for `PortfolioMain`.
///
/// The caller sends `msg.value` of the chain's native token (ETH on Arbitrum,
/// AVAX on Avalanche).  `_from` must equal `msg.sender`; the deposit is
/// credited to the `_from` account on the Dexalot L1 subnet.
///
/// `bridge` selects the cross-chain messaging provider.  Refer to
/// [`BridgeProvider`].
pub const fn make_fn_deposit_native(
    from: Address,
    bridge: BridgeProvider,
) -> [u8; 4 + 32 * 2] {
    concat_arrays!(
        SEL_DEPOSIT_NATIVE,
        leftpad_addr(from),
        leftpad_u8(bridge.as_u8())
    )
}

// ---------------------------------------------------------------------------
// depositToken — deposit ERC-20 token (PortfolioMain, payable)
// ---------------------------------------------------------------------------

/// Encode a Dexalot
/// `depositToken(address _from, bytes32 _symbol, uint256 _quantity, BridgeProvider _bridge)`
/// call for `PortfolioMain`.
///
/// The caller must have ERC-20-approved `PortfolioMain` to transfer
/// `_quantity` of the token identified by `symbol` before sending this
/// calldata.  `_from` must equal `msg.sender`.  The transaction is payable
/// because the bridge fee may be charged in native token; pass the fee as
/// `msg.value`.
///
/// `symbol` is the Dexalot `bytes32` token symbol (e.g. `USDC`, `BTC.b`),
/// left-aligned ASCII zero-padded to 32 bytes.
/// `quantity` is the raw token amount (matching the token's decimals).
/// `bridge` selects the cross-chain messaging provider.
pub const fn make_fn_deposit_token(
    from: Address,
    symbol: Symbol,
    quantity: &U,
    bridge: BridgeProvider,
) -> [u8; 4 + 32 * 4] {
    concat_arrays!(
        SEL_DEPOSIT_TOKEN,
        leftpad_addr(from),
        symbol,
        quantity.0,
        leftpad_u8(bridge.as_u8())
    )
}

// ---------------------------------------------------------------------------
// withdrawNative — withdraw native ALOT (PortfolioSub, Dexalot L1)
// ---------------------------------------------------------------------------

/// Encode a Dexalot `withdrawNative(address payable _to, uint256 _quantity)`
/// call for `PortfolioSub`.
///
/// Withdraws `_quantity` of native ALOT from the caller's subnet balance.
/// The `PortfolioMinter` mints the ALOT to `_to`.  `_to` must equal
/// `msg.sender`.
///
/// `quantity` is in ALOT's 18 decimals.
pub const fn make_fn_withdraw_native(
    to: Address,
    quantity: &U,
) -> [u8; 4 + 32 * 2] {
    concat_arrays!(SEL_WITHDRAW_NATIVE, leftpad_addr(to), quantity.0)
}

// ---------------------------------------------------------------------------
// withdrawToken — withdraw token to a destination mainnet chain (PortfolioSub)
// ---------------------------------------------------------------------------

/// Encode a Dexalot
/// `withdrawToken(address _to, bytes32 _symbol, uint256 _quantity, BridgeProvider _bridge, uint32 _dstChainListOrgChainId)`
/// call for `PortfolioSub`.
///
/// Withdraws `_quantity` of `_symbol` to the caller's address on the
/// destination mainnet chain identified by `dst_chain_list_org_chain_id`.
/// `_to` must equal `msg.sender`.  The function sends a `WITHDRAW` XFER
/// message via the selected bridge; when the message reaches the destination
/// `PortfolioMain`, the locked funds are released to the user's wallet.
///
/// `symbol` is the Dexalot `bytes32` token symbol.
/// `quantity` is the raw token amount.
/// `bridge` selects the cross-chain messaging provider.
/// `dst_chain_list_org_chain_id` is the Dexalot-internal chain id of the
/// destination mainnet (e.g. Avalanche C-Chain, Arbitrum, Base).
pub const fn make_fn_withdraw_token(
    to: Address,
    symbol: Symbol,
    quantity: &U,
    bridge: BridgeProvider,
    dst_chain_list_org_chain_id: u32,
) -> [u8; 4 + 32 * 5] {
    concat_arrays!(
        SEL_WITHDRAW_TOKEN,
        leftpad_addr(to),
        symbol,
        quantity.0,
        leftpad_u8(bridge.as_u8()),
        leftpad_u32(dst_chain_list_org_chain_id)
    )
}

// ---------------------------------------------------------------------------
// withdrawToken (extended) — withdraw with bytes32 destination and options
// ---------------------------------------------------------------------------

/// Encode a Dexalot
/// `withdrawToken(address _from, bytes32 _to, bytes32 _symbol, uint256 _quantity, BridgeProvider _bridge, uint32 _dstChainListOrgChainId, bytes1 _options)`
/// call for `PortfolioSub`.
///
/// Extended withdrawal variant that accepts a `bytes32` destination address
/// (for cross-chain withdrawals to chains with non-EVM address formats) and an
/// `_options` byte.  Setting the `UNWRAP` bit (0x02) in `options` causes the
/// destination `PortfolioMain` to unwrap wrapped native to native before
/// sending.  `_from` must equal `msg.sender`.
///
/// `to` is the `bytes32` destination address on the destination chain.
/// `symbol` is the Dexalot `bytes32` token symbol.
/// `quantity` is the raw token amount.
/// `bridge` selects the cross-chain messaging provider.
/// `dst_chain_list_org_chain_id` is the Dexalot-internal chain id of the
/// destination mainnet.
/// `options` is a `bytes1` bitmask (0 for no options).
pub const fn make_fn_withdraw_token_ext(
    from: Address,
    to: Bytes32Addr,
    symbol: Symbol,
    quantity: &U,
    bridge: BridgeProvider,
    dst_chain_list_org_chain_id: u32,
    options: u8,
) -> [u8; 4 + 32 * 7] {
    concat_arrays!(
        SEL_WITHDRAW_TOKEN_EXT,
        leftpad_addr(from),
        to,
        symbol,
        quantity.0,
        leftpad_u8(bridge.as_u8()),
        leftpad_u32(dst_chain_list_org_chain_id),
        leftpad_u8(options)
    )
}
