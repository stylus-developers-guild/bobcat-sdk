use bobcat_maths::U;

use bobcat_cd::{leftpad_addr, leftpad_bool, leftpad_u32, leftpad_usize};

use crate::selectors;

use array_concat::concat_arrays;

#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;

type Address = [u8; 20];

selectors! {
    SEL_SEND_TOKEN=b"sendToken((uint32,bytes32,uint256,uint256,bytes,bytes,bytes),(uint256,uint256),address)",
    SEL_STARGATE_TYPE = b"stargateType()",
    SEL_QUOTE_OFT = b"quoteOFT((uint32,bytes32,uint256,uint256,bytes,bytes,bytes))",
    SEL_QUOTE_SEND = b"quoteSend((uint32,bytes32,uint256,uint256,bytes,bytes,bytes),bool)",
}

pub const SEND_PARAM_BASE: usize = 10 * 32;

pub const SEND_TOKEN_BASE: usize = 4 + 4 * 32 + SEND_PARAM_BASE;

pub const QUOTE_OFT_BASE: usize = 4 + 1 * 32 + SEND_PARAM_BASE;

pub const QUOTE_SEND_BASE: usize = 4 + 2 * 32 + SEND_PARAM_BASE;

pub const fn make_fn_stargate_type() -> [u8; 4] {
    SEL_STARGATE_TYPE
}

pub const fn make_fn_send_token_slice<
    const EO: usize,
    const EO_PAD: usize,
    const CM: usize,
    const CM_PAD: usize,
    const OC: usize,
    const OC_PAD: usize,
    const ALL: usize,
>(
    dst_eid: u32,
    to: [u8; 32],
    amount_ld: &U,
    min_amount_ld: &U,
    extra_options: [u8; EO],
    compose_msg: [u8; CM],
    oft_cmd: [u8; OC],
    native_fee: &U,
    lz_token_fee: &U,
    refund_address: Address,
) -> [u8; ALL] {
    assert!(
        ALL == SEND_TOKEN_BASE + EO + EO_PAD + CM + CM_PAD + OC + OC_PAD,
        "ALL inconsistent with SEND_TOKEN_BASE + field lengths"
    );
    assert!(
        EO % 32 == 0 || EO + EO_PAD == (EO + 31) & !31,
        "extraOptions padding inconsistent"
    );
    assert!(
        CM % 32 == 0 || CM + CM_PAD == (CM + 31) & !31,
        "composeMsg padding inconsistent"
    );
    assert!(
        OC % 32 == 0 || OC + OC_PAD == (OC + 31) & !31,
        "oftCmd padding inconsistent"
    );
    assert!(
        (ALL - 4) % 32 == 0,
        "total length minus selector must be word-aligned"
    );
    concat_arrays!(
        SEL_SEND_TOKEN,
        leftpad_usize(4 * 32),
        native_fee.0,
        lz_token_fee.0,
        leftpad_addr(refund_address),
        leftpad_u32(dst_eid),
        to,
        amount_ld.0,
        min_amount_ld.0,
        leftpad_usize(7 * 32),
        leftpad_usize(7 * 32 + 32 + EO + EO_PAD),
        leftpad_usize(7 * 32 + 32 + EO + EO_PAD + 32 + CM + CM_PAD),
        leftpad_usize(EO),
        extra_options,
        [0u8; EO_PAD],
        leftpad_usize(CM),
        compose_msg,
        [0u8; CM_PAD],
        leftpad_usize(OC),
        oft_cmd,
        [0u8; OC_PAD]
    )
}

pub const fn make_fn_send_token_no_bytes(
    dst_eid: u32,
    to: [u8; 32],
    amount_ld: &U,
    min_amount_ld: &U,
    native_fee: &U,
    lz_token_fee: &U,
    refund_address: Address,
) -> [u8; SEND_TOKEN_BASE] {
    make_fn_send_token_slice::<0, 0, 0, 0, 0, 0, SEND_TOKEN_BASE>(
        dst_eid,
        to,
        amount_ld,
        min_amount_ld,
        [],
        [],
        [],
        native_fee,
        lz_token_fee,
        refund_address,
    )
}

pub const fn make_fn_quote_oft_slice<
    const EO: usize,
    const EO_PAD: usize,
    const CM: usize,
    const CM_PAD: usize,
    const OC: usize,
    const OC_PAD: usize,
    const ALL: usize,
>(
    dst_eid: u32,
    to: [u8; 32],
    amount_ld: &U,
    min_amount_ld: &U,
    extra_options: [u8; EO],
    compose_msg: [u8; CM],
    oft_cmd: [u8; OC],
) -> [u8; ALL] {
    assert!(ALL == QUOTE_OFT_BASE + EO + EO_PAD + CM + CM_PAD + OC + OC_PAD);
    assert!(EO % 32 == 0 || EO + EO_PAD == (EO + 31) & !31);
    assert!(CM % 32 == 0 || CM + CM_PAD == (CM + 31) & !31);
    assert!(OC % 32 == 0 || OC + OC_PAD == (OC + 31) & !31);
    assert!((ALL - 4) % 32 == 0);
    concat_arrays!(
        SEL_QUOTE_OFT,
        leftpad_usize(1 * 32),
        leftpad_u32(dst_eid),
        to,
        amount_ld.0,
        min_amount_ld.0,
        leftpad_usize(7 * 32),
        leftpad_usize(7 * 32 + 32 + EO + EO_PAD),
        leftpad_usize(7 * 32 + 32 + EO + EO_PAD + 32 + CM + CM_PAD),
        leftpad_usize(EO),
        extra_options,
        [0u8; EO_PAD],
        leftpad_usize(CM),
        compose_msg,
        [0u8; CM_PAD],
        leftpad_usize(OC),
        oft_cmd,
        [0u8; OC_PAD]
    )
}

pub const fn make_fn_quote_oft_no_bytes(
    dst_eid: u32,
    to: [u8; 32],
    amount_ld: &U,
    min_amount_ld: &U,
) -> [u8; QUOTE_OFT_BASE] {
    make_fn_quote_oft_slice::<0, 0, 0, 0, 0, 0, QUOTE_OFT_BASE>(
        dst_eid,
        to,
        amount_ld,
        min_amount_ld,
        [],
        [],
        [],
    )
}

pub const fn make_fn_quote_send_slice<
    const EO: usize,
    const EO_PAD: usize,
    const CM: usize,
    const CM_PAD: usize,
    const OC: usize,
    const OC_PAD: usize,
    const ALL: usize,
>(
    dst_eid: u32,
    to: [u8; 32],
    amount_ld: &U,
    min_amount_ld: &U,
    extra_options: [u8; EO],
    compose_msg: [u8; CM],
    oft_cmd: [u8; OC],
    pay_in_lz_token: bool,
) -> [u8; ALL] {
    assert!(ALL == QUOTE_SEND_BASE + EO + EO_PAD + CM + CM_PAD + OC + OC_PAD);
    assert!(EO % 32 == 0 || EO + EO_PAD == (EO + 31) & !31);
    assert!(CM % 32 == 0 || CM + CM_PAD == (CM + 31) & !31);
    assert!(OC % 32 == 0 || OC + OC_PAD == (OC + 31) & !31);
    assert!((ALL - 4) % 32 == 0);
    concat_arrays!(
        SEL_QUOTE_SEND,
        leftpad_usize(2 * 32),
        leftpad_bool(pay_in_lz_token),
        leftpad_u32(dst_eid),
        to,
        amount_ld.0,
        min_amount_ld.0,
        leftpad_usize(7 * 32),
        leftpad_usize(7 * 32 + 32 + EO + EO_PAD),
        leftpad_usize(7 * 32 + 32 + EO + EO_PAD + 32 + CM + CM_PAD),
        leftpad_usize(EO),
        extra_options,
        [0u8; EO_PAD],
        leftpad_usize(CM),
        compose_msg,
        [0u8; CM_PAD],
        leftpad_usize(OC),
        oft_cmd,
        [0u8; OC_PAD]
    )
}

pub const fn make_fn_quote_send_no_bytes(
    dst_eid: u32,
    to: [u8; 32],
    amount_ld: &U,
    min_amount_ld: &U,
    pay_in_lz_token: bool,
) -> [u8; QUOTE_SEND_BASE] {
    make_fn_quote_send_slice::<0, 0, 0, 0, 0, 0, QUOTE_SEND_BASE>(
        dst_eid,
        to,
        amount_ld,
        min_amount_ld,
        [],
        [],
        [],
        pay_in_lz_token,
    )
}

#[cfg(feature = "alloc")]
fn ceil32(x: usize) -> usize {
    (x + 31) & !31
}

#[cfg(feature = "alloc")]
fn encode_send_param_vec(
    dst_eid: u32,
    to: [u8; 32],
    amount_ld: &U,
    min_amount_ld: &U,
    extra_options: &[u8],
    compose_msg: &[u8],
    oft_cmd: &[u8],
    buf: &mut Vec<u8>,
) {
    let eo_padded = ceil32(extra_options.len());
    let cm_padded = ceil32(compose_msg.len());

    let offset_eo = 7 * 32;
    let offset_cm = offset_eo + 32 + eo_padded;
    let offset_oc = offset_cm + 32 + cm_padded;

    buf.extend_from_slice(&leftpad_u32(dst_eid));
    buf.extend_from_slice(&to);
    buf.extend_from_slice(&amount_ld.0);
    buf.extend_from_slice(&min_amount_ld.0);
    buf.extend_from_slice(&leftpad_usize(offset_eo));
    buf.extend_from_slice(&leftpad_usize(offset_cm));
    buf.extend_from_slice(&leftpad_usize(offset_oc));

    buf.extend_from_slice(&leftpad_usize(extra_options.len()));
    buf.extend_from_slice(extra_options);
    let eo_pad = eo_padded - extra_options.len();
    if eo_pad > 0 {
        buf.extend_from_slice(&[0u8; 32][..eo_pad]);
    }

    buf.extend_from_slice(&leftpad_usize(compose_msg.len()));
    buf.extend_from_slice(compose_msg);
    let cm_pad = cm_padded - compose_msg.len();
    if cm_pad > 0 {
        buf.extend_from_slice(&[0u8; 32][..cm_pad]);
    }

    let oc_padded = ceil32(oft_cmd.len());
    buf.extend_from_slice(&leftpad_usize(oft_cmd.len()));
    buf.extend_from_slice(oft_cmd);
    let oc_pad = oc_padded - oft_cmd.len();
    if oc_pad > 0 {
        buf.extend_from_slice(&[0u8; 32][..oc_pad]);
    }
}

#[cfg(feature = "alloc")]
pub fn make_fn_send_token_vec(
    dst_eid: u32,
    to: [u8; 32],
    amount_ld: &U,
    min_amount_ld: &U,
    extra_options: &[u8],
    compose_msg: &[u8],
    oft_cmd: &[u8],
    native_fee: &U,
    lz_token_fee: &U,
    refund_address: Address,
) -> Vec<u8> {
    let cap = SEND_TOKEN_BASE
        + ceil32(extra_options.len())
        + ceil32(compose_msg.len())
        + ceil32(oft_cmd.len());
    let mut buf = Vec::with_capacity(cap);
    buf.extend_from_slice(&SEL_SEND_TOKEN);
    buf.extend_from_slice(&leftpad_usize(4 * 32));
    buf.extend_from_slice(&native_fee.0);
    buf.extend_from_slice(&lz_token_fee.0);
    buf.extend_from_slice(&leftpad_addr(refund_address));
    encode_send_param_vec(
        dst_eid,
        to,
        amount_ld,
        min_amount_ld,
        extra_options,
        compose_msg,
        oft_cmd,
        &mut buf,
    );
    buf
}

#[cfg(feature = "alloc")]
pub fn make_fn_quote_oft_vec(
    dst_eid: u32,
    to: [u8; 32],
    amount_ld: &U,
    min_amount_ld: &U,
    extra_options: &[u8],
    compose_msg: &[u8],
    oft_cmd: &[u8],
) -> Vec<u8> {
    let cap = QUOTE_OFT_BASE
        + ceil32(extra_options.len())
        + ceil32(compose_msg.len())
        + ceil32(oft_cmd.len());
    let mut buf = Vec::with_capacity(cap);
    buf.extend_from_slice(&SEL_QUOTE_OFT);
    buf.extend_from_slice(&leftpad_usize(1 * 32));
    encode_send_param_vec(
        dst_eid,
        to,
        amount_ld,
        min_amount_ld,
        extra_options,
        compose_msg,
        oft_cmd,
        &mut buf,
    );
    buf
}

#[cfg(feature = "alloc")]
pub fn make_fn_quote_send_vec(
    dst_eid: u32,
    to: [u8; 32],
    amount_ld: &U,
    min_amount_ld: &U,
    extra_options: &[u8],
    compose_msg: &[u8],
    oft_cmd: &[u8],
    pay_in_lz_token: bool,
) -> Vec<u8> {
    let cap = QUOTE_SEND_BASE
        + ceil32(extra_options.len())
        + ceil32(compose_msg.len())
        + ceil32(oft_cmd.len());
    let mut buf = Vec::with_capacity(cap);
    buf.extend_from_slice(&SEL_QUOTE_SEND);
    buf.extend_from_slice(&leftpad_usize(2 * 32));
    buf.extend_from_slice(&leftpad_bool(pay_in_lz_token));
    encode_send_param_vec(
        dst_eid,
        to,
        amount_ld,
        min_amount_ld,
        extra_options,
        compose_msg,
        oft_cmd,
        &mut buf,
    );
    buf
}
