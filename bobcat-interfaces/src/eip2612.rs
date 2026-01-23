use bobcat_maths::U;

use bobcat_cd::{leftpad_addr, leftpad_u8};

use array_concat::concat_arrays;

use crate::selectors;

type Address = [u8; 20];

selectors! {
    SEL_PERMIT = b"permit(address,address,uint256,uint256,uint8,bytes32,bytes32)",
    SEL_NONCES = b"nonces(address)",
    SEL_DOMAIN_SEPARATOR = b"DOMAIN_SEPARATOR()"
}

pub const fn make_fn_permit(
    owner: Address,
    spender: Address,
    value: &U,
    deadline: &U,
    v: u8,
    r: &U,
    s: &U,
) -> [u8; 4 + 32 * 7] {
    concat_arrays!(
        SEL_PERMIT,
        leftpad_addr(owner),
        leftpad_addr(spender),
        value.0,
        deadline.0,
        leftpad_u8(v),
        r.0,
        s.0
    )
}

pub const fn make_fn_nonces(owner: Address) -> [u8; 4 + 32] {
    concat_arrays!(SEL_NONCES, leftpad_addr(owner))
}

pub const fn make_fn_domain_separator() -> [u8; 4] {
    SEL_DOMAIN_SEPARATOR
}
