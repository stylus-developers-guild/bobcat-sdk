//! Narrow Uniswap V4 single-pool swap calldata for Universal Router 2.1.1.
//!
//! The builder targets Universal Router 2.1.1's `V4_SWAP` command and emits an
//! exact-input plan containing `SWAP_EXACT_IN_SINGLE`, `SETTLE_ALL`, and `TAKE_ALL`.
//! Calling `PoolManager.swap` directly is intentionally unsupported: direct callers must
//! implement and settle the PoolManager unlock callback themselves.

use bobcat_maths::U;

use crate::selectors;

pub type Address = [u8; 20];
pub type U24 = [u8; 3];
/// Big-endian two's-complement `int24`.
pub type I24 = [u8; 3];

const COMMAND_V4_SWAP: u8 = 0x10;
const ACTION_SWAP_EXACT_IN_SINGLE: u8 = 0x06;
const ACTION_SETTLE_ALL: u8 = 0x0c;
const ACTION_TAKE_ALL: u8 = 0x0f;

selectors! {
    SEL_EXECUTE = b"execute(bytes,bytes[],uint256)",
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PoolKey {
    /// The numerically smaller currency address; native ETH is the zero address.
    pub currency0: Address,
    /// The numerically larger currency address.
    pub currency1: Address,
    pub fee: U24,
    /// Big-endian two's-complement `int24` tick spacing.
    pub tick_spacing: I24,
    /// Hook contract address, or the zero address for a hookless pool.
    pub hooks: Address,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExactInputSingle<'a> {
    pub pool_key: PoolKey,
    pub zero_for_one: bool,
    pub amount_in: u128,
    pub amount_out_minimum: u128,
    /// Universal Router 2.1.1's optional per-hop minimum output/input price, scaled by 1e36.
    pub min_hop_price_x36: U,
    /// Opaque bytes forwarded unchanged to the pool's hook callbacks.
    pub hook_data: &'a [u8],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EncodeError {
    LengthOverflow,
    BufferTooSmall { required: usize },
}

const PLAN_BASE_LENGTH: usize = 864;
const CALLDATA_BASE_LENGTH: usize = 1124;

fn padded_len(length: usize) -> Option<usize> {
    length.checked_add(31).map(|n| n & !31)
}

/// Required output length for [`make_fn_execute_exact_input_single`].
pub fn exact_input_single_calldata_len(hook_data_len: usize) -> Result<usize, EncodeError> {
    CALLDATA_BASE_LENGTH
        .checked_add(padded_len(hook_data_len).ok_or(EncodeError::LengthOverflow)?)
        .ok_or(EncodeError::LengthOverflow)
}

fn put_usize(output: &mut [u8], offset: usize, value: usize) {
    let bytes = value.to_be_bytes();
    output[offset + 32 - bytes.len()..offset + 32].copy_from_slice(&bytes);
}

fn put_u128(output: &mut [u8], offset: usize, value: u128) {
    output[offset + 16..offset + 32].copy_from_slice(&value.to_be_bytes());
}

fn put_address(output: &mut [u8], offset: usize, value: Address) {
    output[offset + 12..offset + 32].copy_from_slice(&value);
}

fn put_i24(output: &mut [u8], offset: usize, value: I24) {
    if value[0] & 0x80 != 0 {
        output[offset..offset + 29].fill(0xff);
    }
    output[offset + 29..offset + 32].copy_from_slice(&value);
}

/// Encode an exact-input, single-pool V4 swap for Universal Router 2.1.1.
///
/// The generated plan settles at most `amount_in` from the caller and sends at least
/// `amount_out_minimum` back to the caller. ERC-20 input requires the normal Permit2 token
/// approval and Permit2 allowance for the target router. `hook_data` is ABI-encoded as dynamic
/// bytes and forwarded unchanged; `pool_key.hooks` remains the separate hook address.
pub fn make_fn_execute_exact_input_single(
    output: &mut [u8],
    swap: &ExactInputSingle<'_>,
    deadline: &U,
) -> Result<usize, EncodeError> {
    let hook_padded = padded_len(swap.hook_data.len()).ok_or(EncodeError::LengthOverflow)?;
    let required = CALLDATA_BASE_LENGTH
        .checked_add(hook_padded)
        .ok_or(EncodeError::LengthOverflow)?;
    if output.len() < required {
        return Err(EncodeError::BufferTooSmall { required });
    }
    let output = &mut output[..required];
    output.fill(0);

    // UniversalRouter.execute(bytes commands, bytes[] inputs, uint256 deadline)
    output[..4].copy_from_slice(&SEL_EXECUTE);
    put_usize(output, 4, 96);
    put_usize(output, 36, 160);
    output[68..100].copy_from_slice(&deadline.0);
    put_usize(output, 100, 1);
    output[132] = COMMAND_V4_SWAP;
    put_usize(output, 164, 1);
    put_usize(output, 196, 32);

    let plan_len = PLAN_BASE_LENGTH
        .checked_add(hook_padded)
        .ok_or(EncodeError::LengthOverflow)?;
    put_usize(output, 228, plan_len);
    let plan = &mut output[260..260 + plan_len];

    // abi.encode(bytes actions, bytes[] params)
    put_usize(plan, 0, 64);
    put_usize(plan, 32, 128);
    put_usize(plan, 64, 3);
    plan[96..99].copy_from_slice(&[
        ACTION_SWAP_EXACT_IN_SINGLE,
        ACTION_SETTLE_ALL,
        ACTION_TAKE_ALL,
    ]);
    put_usize(plan, 128, 3);
    put_usize(plan, 160, 96);
    put_usize(plan, 192, 512 + hook_padded);
    put_usize(plan, 224, 608 + hook_padded);

    // params[0] = abi.encode(ExactInputSingleParams(...)).
    let swap_param_len = 384 + hook_padded;
    put_usize(plan, 256, swap_param_len);
    let swap_param = &mut plan[288..288 + swap_param_len];
    put_usize(swap_param, 0, 32);
    let tuple = &mut swap_param[32..];
    put_address(tuple, 0, swap.pool_key.currency0);
    put_address(tuple, 32, swap.pool_key.currency1);
    tuple[64 + 29..64 + 32].copy_from_slice(&swap.pool_key.fee);
    put_i24(tuple, 96, swap.pool_key.tick_spacing);
    put_address(tuple, 128, swap.pool_key.hooks);
    tuple[160 + 31] = u8::from(swap.zero_for_one);
    put_u128(tuple, 192, swap.amount_in);
    put_u128(tuple, 224, swap.amount_out_minimum);
    tuple[256..288].copy_from_slice(&swap.min_hop_price_x36.0);
    put_usize(tuple, 288, 320);
    put_usize(tuple, 320, swap.hook_data.len());
    tuple[352..352 + swap.hook_data.len()].copy_from_slice(swap.hook_data);

    let currency_in = if swap.zero_for_one {
        swap.pool_key.currency0
    } else {
        swap.pool_key.currency1
    };
    let currency_out = if swap.zero_for_one {
        swap.pool_key.currency1
    } else {
        swap.pool_key.currency0
    };

    // params[1] = abi.encode(currencyIn, amountIn) for SETTLE_ALL.
    let settle = 672 + hook_padded;
    put_usize(plan, settle, 64);
    put_address(plan, settle + 32, currency_in);
    put_u128(plan, settle + 64, swap.amount_in);

    // params[2] = abi.encode(currencyOut, amountOutMinimum) for TAKE_ALL.
    let take = 768 + hook_padded;
    put_usize(plan, take, 64);
    put_address(plan, take + 32, currency_out);
    put_u128(plan, take + 64, swap.amount_out_minimum);

    Ok(required)
}
