//! DeFi Saver recipe execution calldata builder for Arbitrum.
//!
//! Targets the official `IRecipeExecutor.executeRecipe` ABI from the
//! `defisaver-v3-contracts` repository (commit `ef03f109`):
//!
//! ```text
//! function executeRecipe(StrategyModel.Recipe calldata _currRecipe) external payable;
//! ```
//!
//! The `Recipe` struct is:
//! ```text
//! struct Recipe {
//!     string name;
//!     bytes[] callData;
//!     bytes32[] subData;
//!     bytes4[] actionIds;
//!     uint8[][] paramMapping;
//! }
//! ```
//!
//! Only `executeRecipe` — the primary end-user entry point that runs a recipe
//! directly through the user's wallet — is exposed. `executeRecipeFromStrategy`
//! (bot/subscriber path), `executeRecipeFromTxSaver` (relayer path),
//! `executeActionsFromFL` (flash-loan callback), and all admin/configuration
//! functions are permissioned or secondary flows and are intentionally omitted.
//!
//! Sources:
//! - `IRecipeExecutor.sol` — `contracts/interfaces/core/IRecipeExecutor.sol`
//! - `StrategyModel.sol` — `contracts/core/strategy/StrategyModel.sol`
//! - `RecipeExecutor.sol` — `contracts/core/RecipeExecutor.sol`

use bobcat_cd::leftpad_u8;

use crate::selectors;

/// An EVM address.
pub type Address = [u8; 20];

/// A `bytes4` action identifier.
pub type ActionId = [u8; 4];

/// A `bytes32` sub-data slot.
pub type SubData = [u8; 32];

selectors! {
    SEL_EXECUTE_RECIPE = b"executeRecipe((string,bytes[],bytes32[],bytes4[],uint8[][]))",
}

/// Error returned when calldata cannot be encoded into the supplied buffer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EncodeError {
    /// An arithmetic overflow occurred while computing sizes or offsets.
    LengthOverflow,
    /// The provided output buffer is too small; `required` is the needed size.
    BufferTooSmall { required: usize },
    /// The recipe contains zero actions.
    EmptyActions,
}

/// A single DeFi Saver action within a recipe.
///
/// Each action has a `call_data` blob (opaque input for `IActionBase.executeAction`),
/// an `action_id` (`bytes4(keccak256(ActionName))`), and a `param_mapping` row that
/// describes how inputs are piped from return values or subscription data.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Action<'a> {
    /// Calldata for this action's `executeAction` call.
    pub call_data: &'a [u8],
    /// `bytes4` action identifier registered in the DFS registry.
    pub action_id: ActionId,
    /// Param-mapping row for this action. Each `uint8` maps one input to a source.
    pub param_mapping: &'a [u8],
}

/// A DeFi Saver recipe — a bundle of actions executed sequentially by the
/// `RecipeExecutor`.
///
/// All fields are borrowed: the caller owns the data on the stack or in
/// static storage. No heap allocation is performed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Recipe<'a> {
    /// Human-readable recipe name (used only for logging).
    pub name: &'a [u8],
    /// The actions to execute, in order. Must be non-empty.
    pub actions: &'a [Action<'a>],
    /// Subscription data slots injected from `StrategySub.subData`. May be empty
    /// for manual recipe execution (not part of a strategy subscription).
    pub sub_data: &'a [SubData],
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Round `n` up to the next multiple of 32.
fn padded_len(n: usize) -> Option<usize> {
    n.checked_add(31).map(|v| v & !31)
}

/// Write a `usize` as a big-endian 32-byte word at `offset`.
fn put_usize(output: &mut [u8], offset: usize, value: usize) {
    let bytes = value.to_be_bytes();
    output[offset + 32 - bytes.len()..offset + 32].copy_from_slice(&bytes);
}

/// Size of the `string` ABI encoding: 32 (length) + ceil(len/32)*32.
fn string_enc_len(len: usize) -> Option<usize> {
    Some(32usize + padded_len(len)?)
}

/// Size of a `bytes` element encoding: 32 (length) + ceil(len/32)*32.
fn bytes_elem_len(len: usize) -> Option<usize> {
    string_enc_len(len)
}

/// Size of a `uint8[]` element encoding: 32 (length) + len*32 (one word per uint8).
fn uint8_array_elem_len(len: usize) -> Option<usize> {
    Some(32usize + len.checked_mul(32)?)
}

/// Total size of the `bytes[] callData` field.
fn calldata_array_len(actions: &[Action<'_>]) -> Option<usize> {
    let n = actions.len();
    let mut size = 32usize.checked_add(n.checked_mul(32)?)?; // length + head (offsets)
    for a in actions {
        size = size.checked_add(bytes_elem_len(a.call_data.len())?)?;
    }
    Some(size)
}

/// Total size of the `bytes32[] subData` field.
fn subdata_array_len(sub_data: &[SubData]) -> Option<usize> {
    Some(32usize + sub_data.len().checked_mul(32)?)
}

/// Total size of the `bytes4[] actionIds` field.
fn actionids_array_len(actions: &[Action<'_>]) -> Option<usize> {
    Some(32usize + actions.len().checked_mul(32)?)
}

/// Total size of the `uint8[][] paramMapping` field.
fn parammapping_array_len(actions: &[Action<'_>]) -> Option<usize> {
    let n = actions.len();
    let mut size = 32usize.checked_add(n.checked_mul(32)?)?; // length + head (offsets)
    for a in actions {
        size = size.checked_add(uint8_array_elem_len(a.param_mapping.len())?)?;
    }
    Some(size)
}

// ---------------------------------------------------------------------------
// Public length calculation
// ---------------------------------------------------------------------------

/// Compute the total calldata length for `executeRecipe(recipe)`.
///
/// Use this to size the output buffer before calling [`make_fn_execute_recipe`].
/// Returns `Err(EncodeError::EmptyActions)` if the recipe has no actions.
pub fn execute_recipe_calldata_len(recipe: &Recipe<'_>) -> Result<usize, EncodeError> {
    if recipe.actions.is_empty() {
        return Err(EncodeError::EmptyActions);
    }

    let tuple_head = 5usize * 32;
    let string_size = string_enc_len(recipe.name.len()).ok_or(EncodeError::LengthOverflow)?;
    let calldata_size = calldata_array_len(recipe.actions).ok_or(EncodeError::LengthOverflow)?;
    let subdata_size = subdata_array_len(recipe.sub_data).ok_or(EncodeError::LengthOverflow)?;
    let actionids_size =
        actionids_array_len(recipe.actions).ok_or(EncodeError::LengthOverflow)?;
    let parammapping_size =
        parammapping_array_len(recipe.actions).ok_or(EncodeError::LengthOverflow)?;

    let total = 4usize // selector
        .checked_add(32) // offset to tuple
        .and_then(|n| n.checked_add(tuple_head))
        .and_then(|n| n.checked_add(string_size))
        .and_then(|n| n.checked_add(calldata_size))
        .and_then(|n| n.checked_add(subdata_size))
        .and_then(|n| n.checked_add(actionids_size))
        .and_then(|n| n.checked_add(parammapping_size))
        .ok_or(EncodeError::LengthOverflow)?;

    Ok(total)
}

// ---------------------------------------------------------------------------
// Encoder
// ---------------------------------------------------------------------------

/// Encode `IRecipeExecutor.executeRecipe(Recipe)` into `output`.
///
/// The caller must supply a buffer of at least [`execute_recipe_calldata_len`]
/// bytes. Returns the number of bytes written.
///
/// The recipe must contain at least one action. Callers are responsible for
/// any ERC-20 approvals, flash-loan setup, and sending `msg.value` if the
/// recipe requires native payment.
pub fn make_fn_execute_recipe(
    output: &mut [u8],
    recipe: &Recipe<'_>,
) -> Result<usize, EncodeError> {
    if recipe.actions.is_empty() {
        return Err(EncodeError::EmptyActions);
    }

    let required = execute_recipe_calldata_len(recipe)?;
    if output.len() < required {
        return Err(EncodeError::BufferTooSmall { required });
    }

    let buf = &mut output[..required];
    buf.fill(0);

    // --- Selector + outer offset ---
    buf[..4].copy_from_slice(&SEL_EXECUTE_RECIPE);
    // Offset to tuple = 0x20 (single dynamic arg)
    put_usize(buf, 4, 32);

    // --- Tuple starts at byte 36 (4 + 32) ---
    let tuple_start = 36usize;
    let tuple_head: usize = 5 * 32;

    // Compute field offsets relative to tuple_start
    let off_string = tuple_head;
    let off_calldata = off_string
        .checked_add(string_enc_len(recipe.name.len()).ok_or(EncodeError::LengthOverflow)?)
        .ok_or(EncodeError::LengthOverflow)?;
    let off_subdata = off_calldata
        .checked_add(calldata_array_len(recipe.actions).ok_or(EncodeError::LengthOverflow)?)
        .ok_or(EncodeError::LengthOverflow)?;
    let off_actionids = off_subdata
        .checked_add(subdata_array_len(recipe.sub_data).ok_or(EncodeError::LengthOverflow)?)
        .ok_or(EncodeError::LengthOverflow)?;
    let off_parammapping = off_actionids
        .checked_add(actionids_array_len(recipe.actions).ok_or(EncodeError::LengthOverflow)?)
        .ok_or(EncodeError::LengthOverflow)?;

    // Write tuple head (5 offset words)
    put_usize(buf, tuple_start, off_string);
    put_usize(buf, tuple_start + 32, off_calldata);
    put_usize(buf, tuple_start + 64, off_subdata);
    put_usize(buf, tuple_start + 96, off_actionids);
    put_usize(buf, tuple_start + 128, off_parammapping);

    let n = recipe.actions.len();

    // --- Field 0: string name ---
    {
        let s = tuple_start + off_string;
        put_usize(buf, s, recipe.name.len());
        buf[s + 32..s + 32 + recipe.name.len()].copy_from_slice(recipe.name);
    }

    // --- Field 1: bytes[] callData ---
    {
        let arr = tuple_start + off_calldata;
        put_usize(buf, arr, n); // array length

        // Head: N offset words (relative to head start = arr + 32)
        let head_start = arr + 32;
        let mut elem_offset = n * 32; // first element starts after head
        for (i, action) in recipe.actions.iter().enumerate() {
            put_usize(buf, head_start + i * 32, elem_offset);
            elem_offset = elem_offset
                .checked_add(
                    bytes_elem_len(action.call_data.len()).ok_or(EncodeError::LengthOverflow)?,
                )
                .ok_or(EncodeError::LengthOverflow)?;
        }

        // Tail: each bytes element
        let mut pos = head_start + n * 32;
        for action in recipe.actions.iter() {
            let len = action.call_data.len();
            put_usize(buf, pos, len);
            buf[pos + 32..pos + 32 + len].copy_from_slice(action.call_data);
            let padded = padded_len(len).ok_or(EncodeError::LengthOverflow)?;
            pos = pos
                .checked_add(32)
                .and_then(|p| p.checked_add(padded))
                .ok_or(EncodeError::LengthOverflow)?;
        }
    }

    // --- Field 2: bytes32[] subData ---
    {
        let arr = tuple_start + off_subdata;
        put_usize(buf, arr, recipe.sub_data.len());
        for (i, sd) in recipe.sub_data.iter().enumerate() {
            buf[arr + 32 + i * 32..arr + 32 + i * 32 + 32].copy_from_slice(sd);
        }
    }

    // --- Field 3: bytes4[] actionIds ---
    {
        let arr = tuple_start + off_actionids;
        put_usize(buf, arr, n);
        for (i, action) in recipe.actions.iter().enumerate() {
            // bytes4 is right-padded to 32 bytes (ABI encoding for fixed-size byte arrays)
            let word = rightpad_bytes4(action.action_id);
            buf[arr + 32 + i * 32..arr + 32 + i * 32 + 32].copy_from_slice(&word);
        }
    }

    // --- Field 4: uint8[][] paramMapping ---
    {
        let arr = tuple_start + off_parammapping;
        put_usize(buf, arr, n); // array length

        // Head: N offset words (relative to head start = arr + 32)
        let head_start = arr + 32;
        let mut elem_offset = n * 32; // first element starts after head
        for (i, action) in recipe.actions.iter().enumerate() {
            put_usize(buf, head_start + i * 32, elem_offset);
            elem_offset = elem_offset
                .checked_add(
                    uint8_array_elem_len(action.param_mapping.len())
                        .ok_or(EncodeError::LengthOverflow)?,
                )
                .ok_or(EncodeError::LengthOverflow)?;
        }

        // Tail: each uint8[] element
        let mut pos = head_start + n * 32;
        for action in recipe.actions.iter() {
            let len = action.param_mapping.len();
            put_usize(buf, pos, len);
            // Each uint8 gets its own 32-byte word
            for (j, &val) in action.param_mapping.iter().enumerate() {
                let word = leftpad_u8(val);
                buf[pos + 32 + j * 32..pos + 32 + j * 32 + 32].copy_from_slice(&word);
            }
            let elem_size = len.checked_mul(32).ok_or(EncodeError::LengthOverflow)?;
            pos = pos
                .checked_add(32)
                .and_then(|p| p.checked_add(elem_size))
                .ok_or(EncodeError::LengthOverflow)?;
        }
    }

    Ok(required)
}

/// Right-pad a `bytes4` to a 32-byte word (ABI encoding for fixed bytes).
const fn rightpad_bytes4(arr: [u8; 4]) -> [u8; 32] {
    let mut out = [0u8; 32];
    out[0] = arr[0];
    out[1] = arr[1];
    out[2] = arr[2];
    out[3] = arr[3];
    out
}
