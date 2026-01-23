#![cfg_attr(not(feature = "std"), no_std)]

use core::{
    cmp::{Eq, Ordering},
    fmt::{Debug, Display, Error as FmtError, Formatter, LowerHex, UpperHex},
    ops::{
        Add, AddAssign, BitAnd, BitOr, BitOrAssign, BitXor, Deref, DerefMut, Div, Index, IndexMut,
        Mul, MulAssign, Neg, Not, Rem, Shl, ShlAssign, Shr, ShrAssign, Sub, SubAssign,
    },
    str::FromStr,
};

#[allow(unused)]
use core::ptr::copy_nonoverlapping;

#[cfg(feature = "std")]
use clap::builder::TypedValueParser;

use bobcat_panic::{panic_on_err_div_by_zero, panic_on_err_overflow};

use num_traits::{One, Zero};

#[cfg(feature = "borsh")]
use borsh::{BorshDeserialize, BorshSerialize};

#[cfg(feature = "serde")]
use serde::{Deserialize as SerdeDeserialize, Serialize as SerdeSerialize};

#[cfg(feature = "proptest")]
pub mod strategies;

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(all(
    any(feature = "wasm-bindgen", feature = "wasm-bindgen-wasi"),
    target_arch = "wasm32"
))]
use alloc::boxed::Box;

type Address = [u8; 20];

#[allow(unused)]
use bobcat_host::*;

#[cfg(feature = "ruint-enabled")]
use alloy_primitives::{ruint, U256};

#[cfg(any(
    all(
        feature = "wasm-bindgen-wasi",
        target_os = "wasi",
        any(target_env = "p1", target_env = "p2")
    ),
    all(feature = "wasm-bindgen", target_arch = "wasm32")
))]
use wasm_bindgen::{
    convert::{FromWasmAbi, IntoWasmAbi},
    describe::WasmDescribe,
};

#[cfg(any(
    feature = "alloy-enabled",
    all(not(target_arch = "wasm32"), not(target_arch = "riscv32"))
))]
mod alloy {
    use super::copy_nonoverlapping;

    pub(crate) use alloy_primitives::U256;

    #[cfg(test)]
    pub(crate) use alloy_primitives::I256;

    pub(crate) unsafe fn math_div(out: *mut u8, y: *const u8) {
        unsafe {
            let x = U256::from_be_slice(&*(out as *const [u8; 32]));
            let y = U256::from_be_slice(&*(y as *const [u8; 32]));
            let z = if y.is_zero() {
                // TODO: I think the node returns 0 when this is the case.
                U256::ZERO
            } else {
                x / y
            };
            copy_nonoverlapping(z.to_be_bytes::<32>().as_ptr(), out, 32);
        }
    }

    pub(crate) unsafe fn math_mod(out: *mut u8, y: *const u8) {
        unsafe {
            let x = U256::from_be_slice(&*(out as *const [u8; 32]));
            let y = U256::from_be_slice(&*(y as *const [u8; 32]));
            let z = x % y;
            copy_nonoverlapping(z.to_be_bytes::<32>().as_ptr(), out, 32);
        }
    }

    pub(crate) unsafe fn math_add_mod(a: *mut u8, b: *const u8, c: *const u8) {
        unsafe {
            let x = U256::from_be_slice(&*(a as *const [u8; 32]));
            let y = U256::from_be_slice(&*(b as *const [u8; 32]));
            let z = U256::from_be_slice(&*(c as *const [u8; 32]));
            let x = x.add_mod(y, z);
            copy_nonoverlapping(x.to_be_bytes::<32>().as_ptr(), a, 32);
        }
    }

    pub(crate) unsafe fn math_mul_mod(a: *mut u8, b: *const u8, c: *const u8) {
        unsafe {
            let x = U256::from_be_slice(&*(a as *const [u8; 32]));
            let y = U256::from_be_slice(&*(b as *const [u8; 32]));
            let z = U256::from_be_slice(&*(c as *const [u8; 32]));
            let x = x.mul_mod(y, z);
            copy_nonoverlapping(x.to_be_bytes::<32>().as_ptr(), a, 32);
        }
    }
}

#[cfg(any(
    feature = "alloy-enabled",
    all(not(target_arch = "wasm32"), not(target_arch = "riscv32"))
))]
use alloy::*;

#[derive(Copy, Clone, PartialEq, Hash)]
#[cfg_attr(feature = "proptest", derive(proptest_derive::Arbitrary))]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[cfg_attr(feature = "borsh", derive(BorshDeserialize, BorshSerialize))]
#[cfg_attr(feature = "serde", derive(SerdeSerialize, SerdeDeserialize))]
#[repr(transparent)]
pub struct U(pub [u8; 32]);

#[derive(Copy, Clone, PartialEq, Hash, Debug)]
#[cfg_attr(feature = "proptest", derive(proptest_derive::Arbitrary))]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[cfg_attr(feature = "borsh", derive(BorshDeserialize, BorshSerialize))]
#[cfg_attr(feature = "serde", derive(SerdeSerialize, SerdeDeserialize))]
#[repr(transparent)]
pub struct I(pub [u8; 32]);

#[cfg(feature = "std")]
impl clap::builder::ValueParserFactory for U {
    type Parser = UValueParser;

    fn value_parser() -> Self::Parser {
        UValueParser
    }
}

#[derive(Clone)]
pub struct UValueParser;

#[cfg(feature = "std")]
impl TypedValueParser for UValueParser {
    type Value = U;

    fn parse_ref(
        &self,
        _: &clap::Command,
        _: Option<&clap::Arg>,
        value: &std::ffi::OsStr,
    ) -> Result<Self::Value, clap::Error> {
        let s = value
            .to_str()
            .ok_or_else(|| clap::Error::raw(clap::error::ErrorKind::InvalidUtf8, "bad utf8"))?;
        U::from_str(s).map_err(|e| {
            clap::Error::raw(
                clap::error::ErrorKind::ValueValidation,
                format!("invalid u256: {e}\n"),
            )
        })
    }
}

#[cfg(any(
    all(
        feature = "wasm-bindgen-wasi",
        target_os = "wasi",
        any(target_env = "p1", target_env = "p2")
    ),
    all(feature = "wasm-bindgen", target_arch = "wasm32")
))]
impl WasmDescribe for U {
    fn describe() {
        <Box<[u8]> as WasmDescribe>::describe()
    }
}

#[cfg(any(
    all(
        feature = "wasm-bindgen-wasi",
        target_os = "wasi",
        any(target_env = "p1", target_env = "p2")
    ),
    all(feature = "wasm-bindgen", target_arch = "wasm32")
))]
impl FromWasmAbi for U {
    type Abi = u32;

    #[inline]
    unsafe fn from_abi(js: u32) -> Self {
        let ptr = js as *const u8;
        let mut bytes = [0u8; 32];
        unsafe { copy_nonoverlapping(ptr, bytes.as_mut_ptr(), 32) }
        U(bytes)
    }
}

#[cfg(any(
    all(
        feature = "wasm-bindgen-wasi",
        target_os = "wasi",
        any(target_env = "p1", target_env = "p2")
    ),
    all(feature = "wasm-bindgen", target_arch = "wasm32")
))]
impl IntoWasmAbi for U {
    type Abi = u32;

    #[inline]
    fn into_abi(self) -> u32 {
        let ptr = Box::into_raw(Box::new(self.0)) as *const u8;
        ptr as u32
    }
}

pub fn wrapping_div(x: &U, y: &U) -> U {
    assert!(y.is_some(), "divide by zero");
    let mut b = *x;
    unsafe { math_div(b.as_mut_ptr(), y.as_ptr()) }
    b
}

fn wrapping_div_quo_rem_b<const C: usize>(x: &[u8; C], denom: &[u8; C]) -> ([u8; C], [u8; C]) {
    if denom == &[0u8; C] {
        return ([0u8; C], [0u8; C]);
    }
    let mut q = [0u8; C];
    let mut r = [0u8; C];
    let mut one = [0u8; C];
    one[C - 1] = 1;
    let mut two = [0u8; C];
    two[C - 1] = 2;
    let mut i = 0;
    while i < C * 8 {
        let bit = (x[i / 8] >> (7 - (i % 8))) & 1;
        r = wrapping_mul_b::<C>(&r, &two);
        if bit == 1 {
            r = wrapping_add_b::<C>(&r, &one);
        }
        if r >= *denom {
            r = wrapping_sub_b::<C>(&r, denom);
            q[i / 8] |= 1 << (7 - (i % 8));
        }
        i += 1;
    }
    (q, r)
}

pub fn const_wrapping_div(x: &U, y: &U) -> U {
    U(wrapping_div_quo_rem_b::<32>(&x.0, &y.0).0)
}

#[cfg_attr(test, mutants::skip)]
pub fn checked_div_opt(x: &U, y: &U) -> Option<U> {
    if y.is_zero() {
        None
    } else {
        Some(wrapping_div(x, y))
    }
}

#[cfg_attr(test, mutants::skip)]
pub fn checked_div(x: &U, y: &U) -> U {
    panic_on_err_div_by_zero!(checked_div_opt(x, y), "Division by zero: {x}")
}

pub fn modd(x: &U, y: &U) -> U {
    let mut b = *x;
    unsafe { math_mod(b.as_mut_ptr(), y.as_ptr()) }
    b
}

pub fn mul_mod(mut x: U, y: &U, z: &U) -> U {
    unsafe { math_mul_mod(x.as_mut_ptr(), y.as_ptr(), z.as_ptr()) }
    x
}

const fn wrapping_add_b<const C: usize>(x: &[u8; C], y: &[u8; C]) -> [u8; C] {
    let mut r = [0u8; C];
    let mut c = 0;
    let mut i = C - 1;
    loop {
        let s = x[i] as u16 + y[i] as u16 + c;
        r[i] = s as u8;
        c = s >> 8;
        if i == 0 {
            break;
        }
        i -= 1;
    }
    r
}

pub const fn wrapping_add(x: &U, y: &U) -> U {
    U(wrapping_add_b(&x.0, &y.0))
}

#[cfg_attr(test, mutants::skip)]
pub fn checked_add_opt(x: &U, y: &U) -> Option<U> {
    if x > &(U::MAX - *y) {
        None
    } else {
        let z = x.add_mod(y, &U::MAX);
        if z.is_zero() && (x.is_some() || y.is_some()) {
            Some(U::MAX)
        } else {
            Some(z)
        }
    }
}

#[cfg_attr(test, mutants::skip)]
pub fn checked_add(x: &U, y: &U) -> U {
    panic_on_err_overflow!(checked_add_opt(x, y), "Checked add overflow: {x}, y: {y}")
}

#[cfg_attr(test, mutants::skip)]
pub fn saturating_add(x: &U, y: &U) -> U {
    checked_add_opt(x, y).unwrap_or(U::MAX)
}

const fn wrapping_sub_b<const C: usize>(x: &[u8; C], y: &[u8; C]) -> [u8; C] {
    let mut neg_y = *y;
    let mut i = 0;
    while i < C {
        neg_y[i] = !neg_y[i];
        i += 1;
    }
    let mut c = 1u16;
    let mut i = C - 1;
    loop {
        let sum = neg_y[i] as u16 + c;
        neg_y[i] = sum as u8;
        c = sum >> 8;
        if i == 0 {
            break;
        }
        i -= 1;
    }
    wrapping_add_b(x, &neg_y)
}

pub const fn wrapping_sub(x: &U, y: &U) -> U {
    U(wrapping_sub_b::<32>(&x.0, &y.0))
}

pub fn saturating_sub(x: &U, y: &U) -> U {
    checked_sub_opt(x, y).unwrap_or(U::ZERO)
}

#[cfg_attr(test, mutants::skip)]
pub fn checked_sub_opt(x: &U, y: &U) -> Option<U> {
    if x < y {
        None
    } else {
        Some(wrapping_sub(x, y))
    }
}

#[cfg_attr(test, mutants::skip)]
pub fn checked_sub(x: &U, y: &U) -> U {
    panic_on_err_overflow!(checked_sub_opt(x, y), "Checked sub overflow: {x}, y: {y}")
}

pub const fn wrapping_mul_const_b<const C: usize>(x: &[u8; C], y: &[u8; C]) -> [u8; C] {
    let mut r = [0u8; C];
    let mut i = 0;
    while i < C {
        let mut c = 0u16;
        let mut j = 0;
        while j < C {
            let i_r = i + j;
            if i_r >= C {
                break;
            }
            let r_idx = C - 1 - i_r;
            let xi = x[C - 1 - i] as u16;
            let yj = y[C - 1 - j] as u16;
            let prod = xi * yj + r[r_idx] as u16 + c;
            r[r_idx] = prod as u8;
            c = prod >> 8;
            j += 1;
        }
        i += 1;
    }
    r
}

pub const fn wrapping_mul_const(x: &U, y: &U) -> U {
    U(wrapping_mul_const_b(&x.0, &y.0))
}

pub const fn wrapping_mul_b<const C: usize>(x: &[u8; C], y: &[u8; C]) -> [u8; C] {
    let mut r = [0u8; C];
    let mut i = 0;
    while i < C {
        let mut c = 0u16;
        let mut j = 0;
        while j < C {
            let i_r = i + j;
            if i_r >= C {
                break;
            }
            let r_idx = C - 1 - i_r;
            let xi = x[C - 1 - i] as u16;
            let yj = y[C - 1 - j] as u16;
            let prod = xi * yj + r[r_idx] as u16 + c;
            r[r_idx] = prod as u8;
            c = prod >> 8;
            j += 1;
        }
        i += 1;
    }
    r
}

pub fn wrapping_mul(x: &U, y: &U) -> U {
    U(wrapping_mul_b(&x.0, &y.0))
}

#[cfg_attr(test, mutants::skip)]
pub fn checked_mul_opt(x: &U, y: &U) -> Option<U> {
    if x.is_zero() || y.is_zero() {
        return Some(U::ZERO);
    }
    if x > &(U::MAX / *y) {
        None
    } else {
        let z = x.mul_mod(y, &U::MAX);
        if z.is_zero() {
            Some(U::MAX)
        } else {
            Some(z)
        }
    }
}

pub fn checked_mul(x: &U, y: &U) -> U {
    panic_on_err_overflow!(checked_mul_opt(x, y), "Checked mul overflow: {x}, y: {y}")
}

pub fn saturating_mul(x: &U, y: &U) -> U {
    checked_mul_opt(x, y).unwrap_or(U::MAX)
}

pub fn saturating_div(x: &U, y: &U) -> U {
    checked_div_opt(x, y).unwrap_or(U::MAX)
}

pub fn widening_mul(x: &U, y: &U) -> [u8; 64] {
    let shift_128 = &U([
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0,
    ]);
    let x_hi = x / shift_128;
    let x_lo = x % shift_128;
    let y_hi = y / shift_128;
    let y_lo = y % shift_128;
    let t0 = x_lo.mul_mod(&y_lo, &U::MAX);
    let t1 = x_hi.mul_mod(&y_lo, &U::MAX);
    let t2 = x_lo.mul_mod(&y_hi, &U::MAX);
    let t3 = x_hi.mul_mod(&y_hi, &U::MAX);
    let t0_hi = &t0 / shift_128;
    let t0_lo = &t0 % shift_128;
    let t1_hi = &t1 / shift_128;
    let t1_lo = &t1 % shift_128;
    let t2_hi = &t2 / shift_128;
    let t2_lo = &t2 % shift_128;
    let mid = (t0_hi + t1_lo) + t2_lo;
    let mid_hi = &mid / shift_128;
    let mid_lo = &mid % shift_128;
    let mid_lo_shifted = mid_lo.mul_mod(shift_128, &U::MAX);
    let out_low = t0_lo + mid_lo_shifted;
    let out_high = t3 + t1_hi + t2_hi + mid_hi;
    let mut o = [0u8; 64];
    o[..32].copy_from_slice(&out_high.0);
    o[32..].copy_from_slice(&out_low.0);
    o
}

/// Widening then truncate mul div that's cheaper in codesize that's safe for cold
/// operations. The most expensive in gas costs.
pub fn widening_mul_div(x: &U, y: &U, denom: U) -> Option<(U, bool)> {
    if denom.is_zero() {
        return None;
    }
    if x.is_zero() {
        return Some((U::ZERO, false));
    }
    // We use a boring method if the overflow wouldn't happen:
    if wrapping_div(&U::MAX, x) >= *y {
        let l = wrapping_mul(x, y);
        let carry = x.mul_mod(y, &denom).is_some();
        return Some((wrapping_div(&l, &denom), carry));
    }
    let x = widening_mul(x, y);
    let mut d = [0u8; 64];
    d[32..].copy_from_slice(&denom.0);
    let (q, rem) = wrapping_div_quo_rem_b::<64>(&x, &d);
    if q[..32] != [0u8; 32] {
        return None;
    }
    let l: [u8; 32] = q[32..].try_into().unwrap();
    let l = U::from(l);
    let has_carry = rem[32..] != [0u8; 32];
    Some((l, has_carry))
}

pub fn widening_mul_div_round_up(x: &U, y: &U, denom: U) -> Option<U> {
    let (x, y) = widening_mul_div(x, y, denom)?;
    if x.is_max() && y {
        return None;
    }
    Some(if y { x + U::ONE } else { x })
}

/// Muldiv that's used in practice by Uniswap and other on-chain dapps.
/// Middling in gas costs.
pub fn mul_div(x: &U, y: &U, mut denom: U) -> Option<(U, bool)> {
    // Implemented from https://xn--2-umb.com/21/muldiv/
    if denom.is_zero() {
        return None;
    }
    if x.is_zero() {
        return Some((U::ZERO, false));
    }
    let mut prod0 = wrapping_mul(x, y);
    let mm = mul_mod(*x, y, &U::MAX);
    let mut prod1 = wrapping_sub(
        &wrapping_sub(&mm, &prod0),
        &if prod0 > mm { U::ONE } else { U::ZERO },
    );
    if prod1.is_zero() {
        let carry = mul_mod(*x, y, &denom).is_some();
        return Some((wrapping_div(&prod0, &denom), carry));
    }
    if prod1 >= denom {
        return None;
    }
    let remainder = mul_mod(*x, y, &denom);
    let carry = remainder.is_some();
    if remainder > prod0 {
        prod1 -= U::ONE;
    }
    prod0 = wrapping_sub(&prod0, &remainder);
    let mut twos = wrapping_sub(&U::ZERO, &denom) & denom;
    denom = wrapping_div(&denom, &twos);
    prod0 = wrapping_div(&prod0, &twos);
    twos = wrapping_add(
        &wrapping_div(&wrapping_sub(&U::ZERO, &twos), &twos),
        &U::ONE,
    );
    prod0 = prod0 | wrapping_mul(&prod1, &twos);
    let mut inv = wrapping_mul(&U::from(3u32), &denom) ^ U::from(2u32);
    for _ in 0..6 {
        inv = wrapping_mul(
            &inv,
            &wrapping_sub(&U::from(2u32), &wrapping_mul(&denom, &inv)),
        );
    }
    Some((wrapping_mul(&prod0, &inv), carry))
}

pub fn mul_div_round_up(x: &U, y: &U, denom_and_rem: U) -> Option<U> {
    let (x, y) = mul_div(x, y, denom_and_rem)?;
    if x.is_max() && y {
        return None;
    }
    Some(if y { x + U::ONE } else { x })
}

/// The cheapest muldiv operation, but the most expensive in codesize muldiv.
#[cfg(feature = "ruint-enabled")]
pub fn ruint_mul_div(x: &U, y: &U, denom: U) -> Option<(U, bool)> {
    if denom.is_zero() {
        return None;
    }
    let x = U256::from_be_slice(x.as_slice());
    let y = U256::from_be_slice(y.as_slice());
    let mut denom = U256::from_be_slice(denom.as_slice());
    let mut mul_and_quo = x.widening_mul::<256, 4, 512, 8>(y);
    unsafe {
        ruint::algorithms::div(mul_and_quo.as_limbs_mut(), denom.as_limbs_mut());
    }
    let limbs = mul_and_quo.into_limbs();
    if limbs[4..] != [0_u64; 4] {
        return None;
    }
    let has_carry = !denom.is_zero();
    let r = U(U256::from_limbs_slice(&limbs[0..4]).to_be_bytes::<32>());
    Some((r, has_carry))
}

#[cfg(feature = "ruint-enabled")]
pub fn ruint_mul_div_round_up(x: &U, y: &U, denom: U) -> Option<U> {
    let (x, y) = ruint_mul_div(x, y, denom)?;
    if x.is_max() && y {
        return None;
    }
    Some(if y { x + U::ONE } else { x })
}

/// Rooti iterative method based on the 9lives implementation. Using this
/// operation is the equivalent of pow(x, 1/n).
pub fn checked_rooti(x: U, n: u32) -> Option<U> {
    if n == 0 {
        return None;
    }
    if x.is_zero() {
        return Some(U::ZERO);
    }
    if n == 1 {
        return Some(x);
    }
    // Due to the nature of this iterative method, we hardcode some
    // values to have consistency with the 9lives reference.
    if x == U::from(4u32) && n == 2 {
        return Some(U::from(2u32));
    }
    let n_u256 = U::from(n);
    let n_1 = n_u256 - U::ONE;
    // Initial guess: 2^ceil(bits(x)/n)
    let mut b = 0;
    let mut t = x;
    while t.is_some() {
        b += 1;
        t >>= 1;
    }
    let shift = (b + n as usize - 1) / n as usize;
    let mut z = U::ONE << shift;
    let mut y = x;
    // Newton's method:
    while z < y {
        y = z;
        let p = z.checked_pow(&n_1)?;
        z = ((x / p) + (z * n_1)) / n_u256;
    }
    // Correct overshoot:
    if y.checked_pow(&n_u256)? > x {
        y -= U::ONE;
    }
    Some(y)
}

pub fn wrapping_pow(x: &U, exp: &U) -> U {
    let mut r = U::ONE;
    let mut i = U::ZERO;
    while &i < exp {
        r = wrapping_mul(&r, x);
        i += U::ONE;
    }
    r
}

pub fn checked_pow(x: &U, exp: &U) -> Option<U> {
    let mut r = U::ONE;
    let mut i = U::ZERO;
    while &i < exp {
        r = checked_mul_opt(&r, x)?;
        i += U::ONE;
    }
    Some(r)
}

impl Add for U {
    type Output = U;

    fn add(self, rhs: U) -> U {
        cfg_if::cfg_if! {
            if #[cfg(debug_assertions)] {
                checked_add_opt(&self, &rhs).expect("overflow when add")
            } else {
                wrapping_add(&self, &rhs)
            }
        }
    }
}

impl Add for &U {
    type Output = U;

    fn add(self, rhs: &U) -> U {
        cfg_if::cfg_if! {
            if #[cfg(debug_assertions)] {
                checked_add_opt(self, rhs).expect("overflow when add")
            } else {
                wrapping_add(self, rhs)
            }
        }
    }
}

impl AddAssign for U {
    fn add_assign(&mut self, o: Self) {
        *self = *self + o;
    }
}

impl Sub for U {
    type Output = U;

    fn sub(self, rhs: U) -> U {
        cfg_if::cfg_if! {
            if #[cfg(debug_assertions)] {
                checked_sub_opt(&self, &rhs).expect("overflow when sub")
            } else {
                wrapping_sub(&self, &rhs)
            }
        }
    }
}

impl Sub for &U {
    type Output = U;

    fn sub(self, rhs: &U) -> U {
        cfg_if::cfg_if! {
            if #[cfg(debug_assertions)] {
                checked_sub_opt(self, rhs).expect("overflow when sub")
            } else {
                wrapping_sub(self, rhs)
            }
        }
    }
}

impl SubAssign for U {
    fn sub_assign(&mut self, o: Self) {
        *self = *self - o;
    }
}

impl Mul for U {
    type Output = U;

    fn mul(self, rhs: U) -> U {
        cfg_if::cfg_if! {
            if #[cfg(debug_assertions)] {
                checked_mul_opt(&self, &rhs).expect("overflow when mul")
            } else {
                wrapping_mul(&self, &rhs)
            }
        }
    }
}

impl Mul for &U {
    type Output = U;

    fn mul(self, rhs: &U) -> U {
        cfg_if::cfg_if! {
            if #[cfg(debug_assertions)] {
                checked_mul_opt(self, rhs).expect("overflow when mul")
            } else {
                wrapping_mul(self, rhs)
            }
        }
    }
}

impl MulAssign for U {
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs
    }
}

impl Div for U {
    type Output = U;

    fn div(self, rhs: U) -> U {
        cfg_if::cfg_if! {
            if #[cfg(debug_assertions)] {
                checked_div_opt(&self, &rhs).expect("overflow when div")
            } else {
                wrapping_div(&self, &rhs)
            }
        }
    }
}

impl Div for &U {
    type Output = U;

    fn div(self, rhs: &U) -> U {
        cfg_if::cfg_if! {
            if #[cfg(debug_assertions)] {
                checked_div_opt(self, rhs).expect("overflow when div")
            } else {
                wrapping_div(self, rhs)
            }
        }
    }
}

impl Rem for U {
    type Output = U;

    fn rem(self, rhs: U) -> U {
        modd(&self, &rhs)
    }
}

impl Rem for &U {
    type Output = U;

    fn rem(self, rhs: &U) -> U {
        modd(self, rhs)
    }
}

impl Shl<usize> for U {
    type Output = Self;

    fn shl(self, shift: usize) -> Self::Output {
        if shift >= 256 {
            return U::ZERO;
        }
        let mut result = [0u8; 32];
        let byte_shift = shift / 8;
        let bit_shift = shift % 8;
        if bit_shift == 0 {
            for i in 0..(32 - byte_shift) {
                result[i] = self.0[i + byte_shift];
            }
        } else {
            let mut carry = 0u8;
            for i in (byte_shift..32).rev() {
                let src_idx = i;
                let dst_idx = i - byte_shift;
                let byte = self.0[src_idx];
                result[dst_idx] = (byte << bit_shift) | carry;
                carry = byte >> (8 - bit_shift);
            }
        }
        U(result)
    }
}

impl ShlAssign<usize> for U {
    fn shl_assign(&mut self, rhs: usize) {
        *self = *self << rhs
    }
}

impl BitAnd for U {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        let mut r = U::ZERO;
        for i in 0..32 {
            r[i] = self[i] & rhs[i];
        }
        r
    }
}

impl BitOr for U {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        let mut r = U::ZERO;
        for i in 0..32 {
            r[i] = self[i] | rhs[i];
        }
        r
    }
}

impl BitXor for U {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
        let mut r = U::ZERO;
        for i in 0..32 {
            r[i] = self[i] ^ rhs[i];
        }
        r
    }
}

impl BitOrAssign for U {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = *self | rhs
    }
}

impl Shr<usize> for U {
    type Output = Self;

    fn shr(self, shift: usize) -> Self::Output {
        if shift >= 256 {
            return U::ZERO;
        }
        let mut result = U::ZERO;
        let byte_shift = shift / 8;
        let bit_shift = shift % 8;
        if bit_shift == 0 {
            for i in byte_shift..32 {
                result[i] = self.0[i - byte_shift];
            }
        } else {
            let mut carry = 0u8;
            for i in 0..(32 - byte_shift) {
                let src_idx = i;
                let dst_idx = i + byte_shift;
                let byte = self.0[src_idx];
                result[dst_idx] = (byte >> bit_shift) | carry;
                carry = byte << (8 - bit_shift);
            }
        }
        result
    }
}

impl ShrAssign<usize> for U {
    fn shr_assign(&mut self, rhs: usize) {
        *self = *self >> rhs
    }
}

impl Eq for U {}

impl PartialOrd for U {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for U {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl LowerHex for U {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        let mut b = [0u8; 32 * 2];
        let s = const_hex::encode_to_str(self.0, &mut b).unwrap();
        write!(f, "{s}")
    }
}

impl UpperHex for U {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        let mut b = [0u8; 32 * 2];
        let s = const_hex::encode_to_str_upper(self.0, &mut b).unwrap();
        write!(f, "{s}")
    }
}

impl Debug for U {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        write!(f, "{self:x}")
    }
}

impl Not for U {
    type Output = Self;

    fn not(mut self) -> Self::Output {
        for i in 0..32 {
            self[i] = !self[i]
        }
        self
    }
}

impl Neg for U {
    type Output = Self;

    fn neg(self) -> Self {
        let mut r = U::ZERO;
        let mut carry = 1u16;
        for i in (0..32).rev() {
            let inverted = !self.0[i] as u16;
            let sum = inverted + carry;
            r[i] = sum as u8;
            carry = sum >> 8;
        }
        r
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum UFromStrErr {
    InvalidChar(char),
    Overflow,
    Empty,
}

impl Display for UFromStrErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl core::error::Error for UFromStrErr {}

impl FromStr for U {
    type Err = UFromStrErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(UFromStrErr::Empty);
        }
        let mut r = U::ZERO;
        for c in s.chars() {
            r *= U::from_u32(10);
            r += match c {
                '0'..='9' => U::from(c as u8 - b'0'),
                _ => return Err(UFromStrErr::InvalidChar(c)),
            };
        }
        Ok(r)
    }
}

impl U {
    pub const ZERO: Self = U([0u8; 32]);

    pub const MAX: Self = U([u8::MAX; 32]);

    pub const ONE: Self = U([
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 1,
    ]);

    pub fn is_true(&self) -> bool {
        self.0[31] == 1
    }

    pub const fn is_zero(&self) -> bool {
        let mut i = 0;
        while i < 32 {
            if self.0[i] != 0 {
                return false;
            }
            i += 1;
        }
        true
    }

    pub fn abs_diff(&self, y: &U) -> U {
        if self > y {
            self - y
        } else {
            y - self
        }
    }

    pub const fn const_addr(self) -> Address {
        self.const_20_slice()
    }

    pub const fn is_max(&self) -> bool {
        let mut i = 0;
        while i < 32 {
            if self.0[i] != u8::MAX {
                return false;
            }
            i += 1;
        }
        true
    }

    pub fn is_some(&self) -> bool {
        !self.is_zero()
    }

    pub fn trailing_zeros(&self) -> usize {
        let mut count = 0;
        for i in (0..32).rev() {
            if self[i] == 0 {
                count += 8;
            } else {
                count += self[i].trailing_zeros() as usize;
                break;
            }
        }
        count
    }

    pub fn as_slice(&self) -> &[u8; 32] {
        &self.0
    }

    pub const fn from_slice_leftpad(x: &[u8]) -> Option<U> {
        if x.len() > 32 {
            return None;
        }
        let mut b = [0u8; 32];
        let mut i = 0;
        while i < x.len() {
            b[32 - x.len() + i] = x[i];
            i += 1;
        }
        Some(U(b))
    }

    #[cfg(feature = "alloc")]
    pub fn as_vec(self) -> alloc::vec::Vec<u8> {
        self.0.to_vec()
    }

    pub fn checked_add_opt(&self, y: &Self) -> Option<Self> {
        checked_add_opt(self, y)
    }

    pub fn checked_add(&self, y: &Self) -> Self {
        checked_add(self, y)
    }

    pub fn checked_mul_opt(&self, y: &Self) -> Option<Self> {
        checked_mul_opt(self, y)
    }

    pub fn checked_mul(&self, y: &Self) -> Self {
        checked_mul(self, y)
    }

    pub fn checked_sub_opt(&self, y: &Self) -> Option<Self> {
        checked_sub_opt(self, y)
    }

    pub fn checked_sub(&self, y: &Self) -> Self {
        checked_sub(self, y)
    }

    pub fn checked_div_opt(&self, y: &Self) -> Option<Self> {
        checked_div_opt(self, y)
    }

    pub fn checked_div(&self, y: &Self) -> Self {
        checked_div(self, y)
    }

    pub fn checked_pow(&self, exp: &U) -> Option<Self> {
        checked_pow(self, exp)
    }

    pub fn wrapping_add(&self, y: &Self) -> U {
        wrapping_add(self, y)
    }

    pub fn wrapping_sub(&self, y: &Self) -> U {
        wrapping_sub(self, y)
    }

    pub fn wrapping_mul(&self, y: &Self) -> U {
        wrapping_mul(self, y)
    }

    pub fn wrapping_div(&self, y: &Self) -> U {
        wrapping_div(self, y)
    }

    pub fn saturating_add(&self, y: &Self) -> U {
        saturating_add(self, y)
    }

    pub fn saturating_sub(&self, y: &Self) -> U {
        saturating_sub(self, y)
    }

    pub fn saturating_mul(&self, y: &Self) -> U {
        saturating_mul(self, y)
    }

    pub fn saturating_div(&self, y: &Self) -> Self {
        saturating_div(self, y)
    }

    pub fn wrapping_neg(self) -> Self {
        let mut x = self;
        let mut carry = 1u8;
        for b in x.iter_mut().rev() {
            *b = (!*b).wrapping_add(carry);
            carry = b.is_zero() as u8;
        }
        x
    }

    pub fn mul_div(&self, y: &Self, z: Self) -> Option<(Self, bool)> {
        mul_div(self, y, z)
    }

    pub fn mul_div_round_up(&self, y: &Self, z: Self) -> Option<Self> {
        mul_div_round_up(self, y, z)
    }

    pub fn widening_mul_div(&self, y: &Self, z: Self) -> Option<(Self, bool)> {
        widening_mul_div(self, y, z)
    }

    pub fn widening_mul_div_round_up(&self, y: &Self, z: Self) -> Option<Self> {
        widening_mul_div_round_up(self, y, z)
    }

    #[cfg(feature = "ruint-enabled")]
    pub fn ruint_mul_div(&self, y: &Self, z: Self) -> Option<(Self, bool)> {
        ruint_mul_div(self, y, z)
    }

    #[cfg(feature = "ruint-enabled")]
    pub fn ruint_mul_div_round_up(&self, y: &Self, z: Self) -> Option<Self> {
        ruint_mul_div_round_up(self, y, z)
    }

    pub fn mul_mod(&self, y: &Self, z: &Self) -> Self {
        mul_mod(*self, y, z)
    }

    pub fn add_mod(&self, y: &Self, z: &Self) -> Self {
        let mut b = self.0;
        unsafe { math_add_mod(b.as_mut_ptr(), y.as_ptr(), z.as_ptr()) }
        Self(b)
    }

    pub fn checked_rooti(self, x: u32) -> Option<Self> {
        checked_rooti(self, x)
    }

    pub fn from_hex(x: &str) -> Option<U> {
        match const_hex::decode_to_array::<_, 32>(x) {
            Ok(v) => Some(U(v)),
            Err(_) => None,
        }
    }

    pub const fn const_from_hex(x: &[u8; 64]) -> Option<U> {
        match const_hex::const_decode_to_array::<32>(x) {
            Ok(v) => Some(U(v)),
            Err(_) => None,
        }
    }
}

impl Display for U {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        if self.is_zero() {
            return write!(f, "0");
        }
        let mut result = [0u8; 78];
        let mut i = 0;
        for byte in self.0 {
            let mut carry = byte as u32;
            for digit in result[..i].iter_mut() {
                let temp = (*digit as u32) * 256 + carry;
                *digit = (temp % 10) as u8;
                carry = temp / 10;
            }
            while carry > 0 {
                result[i] = (carry % 10) as u8;
                i += 1;
                debug_assert!(78 >= i, "{} > {i}", result.len());
                carry /= 10;
            }
        }
        for &digit in result[..i].iter().rev() {
            write!(f, "{}", digit)?;
        }
        Ok(())
    }
}

impl From<U> for [u8; 32] {
    fn from(x: U) -> Self {
        x.0
    }
}

impl From<&U> for U {
    fn from(x: &U) -> Self {
        *x
    }
}

impl From<U> for bool {
    fn from(x: U) -> Self {
        x.0[31] == 1
    }
}

impl From<&[u8]> for U {
    fn from(x: &[u8]) -> Self {
        let x: &[u8; 32] = x.try_into().unwrap();
        (*x).into()
    }
}

impl From<&[u8; 32]> for &U {
    fn from(x: &[u8; 32]) -> Self {
        unsafe { &*(x as *const [u8; 32] as *const U) }
    }
}

impl From<[u8; 32]> for U {
    fn from(x: [u8; 32]) -> Self {
        U(x)
    }
}

impl Deref for U {
    type Target = [u8; 32];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for U {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<bool> for U {
    fn from(x: bool) -> Self {
        U::from(&[x as u8])
    }
}

impl Zero for U {
    fn zero() -> Self {
        U::ZERO
    }

    fn is_zero(&self) -> bool {
        self.0.iter().all(|&b| b == 0)
    }
}

impl Default for U {
    fn default() -> Self {
        U::ZERO
    }
}

impl One for U {
    fn one() -> Self {
        U::ONE
    }
}

impl Index<usize> for U {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl IndexMut<usize> for U {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

impl I {
    fn is_neg(&self) -> bool {
        self.0[0] & 0x80 != 0
    }

    pub fn is_zero(&self) -> bool {
        *self == Self::ZERO
    }

    pub fn is_some(&self) -> bool {
        !self.is_zero()
    }

    pub fn as_slice(&self) -> &[u8; 32] {
        &self.0
    }

    fn neg(&self) -> Self {
        let x = wrapping_add(&U(self.0.map(|b| !b)), &U::ONE);
        I(x.0)
    }

    fn abs(self) -> U {
        if self.is_neg() {
            U(self.neg().0)
        } else {
            U(self.0)
        }
    }
}

macro_rules! from_slices {
    ($($n:expr),+ $(,)?) => {
        $(
            paste::paste! {
                impl From<&[u8; $n]> for U {
                    fn from(x: &[u8; $n]) -> Self {
                        let mut b = [0u8; 32];
                        b[32 - $n..].copy_from_slice(x);
                        U(b)
                    }
                }

                impl From<[u8; $n]> for U {
                    fn from(x: [u8; $n]) -> Self {
                        U::from(&x)
                    }
                }

                impl U {
                    pub const fn [<const_ $n _slice>](self) -> [u8; $n] {
                        let mut b = [0u8; $n];
                        let mut i = 0;
                        while i < $n {
                            b[i] = self.0[32-$n+i];
                            i += 1;
                        }
                        b
                    }
                }

                impl From<U> for [u8; $n] {
                    fn from(x: U) -> Self {
                        unsafe { *(x.as_ptr().add(32 - $n) as *const [u8; $n]) }
                    }
                }
            }
        )+
    };
}

from_slices!(
    1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26,
    27, 28, 29, 30, 31
);

impl From<&U> for Address {
    fn from(x: &U) -> Self {
        (*x).into()
    }
}

macro_rules! from_ints {
    ($($t:ty),+ $(,)?) => {
        $(
            paste::paste! {
                impl U {
                    pub const fn [<from_ $t>](x: $t) -> U {
                        U(array_concat::concat_arrays!(
                            [0u8; 32-core::mem::size_of::<$t>()],
                            x.to_be_bytes())
                        )
                    }
                }

                impl From<$t> for U {
                    fn from(x: $t) -> Self {
                        U::[<from_ $t>](x)
                    }
                }

                impl From<U> for $t {
                    fn from(x: U) -> Self {
                        Self::from_be_bytes(x.into())
                    }
                }
            }
        )+
    };
}

#[macro_export]
macro_rules! u {
    ($e:expr) => {
        $crate::U::from_u32($e)
    };
}

from_ints! { u8, u16, u32, u64, u128, usize }

impl From<I> for [u8; 32] {
    fn from(x: I) -> Self {
        x.0
    }
}

impl From<[u8; 32]> for I {
    fn from(x: [u8; 32]) -> Self {
        I(x)
    }
}

fn i_add(x: &I, y: &I) -> I {
    I(wrapping_add(&U(x.0), &U(y.0)).0)
}

fn i_sub(x: &I, y: &I) -> I {
    I(wrapping_sub(&U(x.0), &U(y.0)).0)
}

fn i_mul(x: &I, y: &I) -> I {
    let result = wrapping_mul(&U(x.0), &U(y.0));
    I(result.0)
}

fn i_div(x: &I, y: &I) -> I {
    let r = wrapping_div(&x.abs(), &y.abs());
    if x.is_neg() ^ y.is_neg() {
        I(r.0).neg()
    } else {
        I(r.0)
    }
}

fn i_rem(x: &I, y: &I) -> I {
    let r = modd(&x.abs(), &y.abs());
    if x.is_neg() {
        I(r.0).neg()
    } else {
        I(r.0)
    }
}

impl Add for I {
    type Output = I;
    fn add(self, rhs: I) -> I {
        i_add(&self, &rhs)
    }
}

impl Add for &I {
    type Output = I;
    fn add(self, rhs: &I) -> I {
        i_add(self, rhs)
    }
}

impl Sub for I {
    type Output = I;
    fn sub(self, rhs: I) -> I {
        i_sub(&self, &rhs)
    }
}

impl Sub for &I {
    type Output = I;
    fn sub(self, rhs: &I) -> I {
        i_sub(self, rhs)
    }
}

impl Mul for I {
    type Output = I;
    fn mul(self, rhs: I) -> I {
        i_mul(&self, &rhs)
    }
}

impl Mul for &I {
    type Output = I;
    fn mul(self, rhs: &I) -> I {
        i_mul(self, rhs)
    }
}

impl Div for I {
    type Output = I;
    fn div(self, rhs: I) -> I {
        i_div(&self, &rhs)
    }
}

impl Div for &I {
    type Output = I;
    fn div(self, rhs: &I) -> I {
        i_div(self, rhs)
    }
}

impl Rem for I {
    type Output = I;
    fn rem(self, rhs: I) -> I {
        i_rem(&self, &rhs)
    }
}

impl Eq for I {}

impl PartialOrd for I {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for I {
    fn cmp(&self, other: &Self) -> Ordering {
        let self_sign = self.0[0] & 0x80;
        let other_sign = other.0[0] & 0x80;
        match (self_sign, other_sign) {
            (0, 0x80) => Ordering::Greater,
            (0x80, 0) => Ordering::Less,
            _ => self.0.cmp(&other.0),
        }
    }
}

impl Rem for &I {
    type Output = I;
    fn rem(self, rhs: &I) -> I {
        i_rem(self, rhs)
    }
}

impl I {
    pub const ZERO: Self = I([0u8; 32]);

    pub const ONE: Self = I([
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 1,
    ]);
}

impl Zero for I {
    fn zero() -> Self {
        I::ZERO
    }
    fn is_zero(&self) -> bool {
        self.0.iter().all(|&b| b == 0)
    }
}

impl Default for I {
    fn default() -> Self {
        I::ZERO
    }
}

impl One for I {
    fn one() -> Self {
        I::ONE
    }
}

#[test]
fn test_is_zeroes() {
    assert!(U::ZERO.is_zero());
    assert!(U::ONE.is_some());
    assert!(I::ZERO.is_zero());
    assert!(I::ONE.is_some());
}

#[cfg(all(
    test,
    feature = "alloy-enabled",
    feature = "proptest",
    feature = "std",
    not(target_arch = "wasm32")
))]
mod test {
    use proptest::prelude::*;

    use super::*;

    fn strat_any_u256() -> impl Strategy<Value = U256> {
        // Arbitrary seems to be having some issues with U256:
        any::<[u8; 32]>().prop_map(U256::from_be_bytes)
    }

    proptest! {
        #[test]
        fn wrapping_div_b_zero_denominator_yields_zero(numerator in any::<[u8; 4]>()) {
            let zero = [0u8; 4];
            prop_assert_eq!(wrapping_div_quo_rem_b::<4>(&numerator, &zero).0, zero);
        }

        #[test]
        fn wrapping_div_b_matches_integer_division(
            numerator in any::<[u8; 4]>(),
            denominator in any::<[u8; 4]>().prop_filter("denominator must be non-zero", |d| *d != [0u8; 4])
        ) {
            let numerator_u32 = u32::from_be_bytes(numerator);
            let denominator_u32 = u32::from_be_bytes(denominator);
            let expected = numerator_u32 / denominator_u32;
            prop_assert_eq!(
                wrapping_div_quo_rem_b::<4>(&numerator, &denominator).0,
                expected.to_be_bytes()
            );
        }

        #[test]
        fn wrapping_mod_b_matches_integer_modulo(
            numerator in any::<[u8; 4]>(),
            denominator in any::<[u8; 4]>().prop_filter("denominator must be non-zero", |d| *d != [0u8; 4])
        ) {
            let numerator_u32 = u32::from_be_bytes(numerator);
            let denominator_u32 = u32::from_be_bytes(denominator);
            let expected = numerator_u32 % denominator_u32;
            prop_assert_eq!(
                wrapping_div_quo_rem_b::<4>(&numerator, &denominator).1,
                expected.to_be_bytes()
            );
        }

        #[test]
        fn wrapping_add_b_handles_carry(lhs in any::<[u8; 4]>(), rhs in any::<[u8; 4]>()) {
            let lhs_u32 = u32::from_be_bytes(lhs);
            let rhs_u32 = u32::from_be_bytes(rhs);
            let expected = lhs_u32.wrapping_add(rhs_u32);
            prop_assert_eq!(wrapping_add_b::<4>(&lhs, &rhs), expected.to_be_bytes());
        }

        #[test]
        fn wrapping_sub_b_handles_borrow(lhs in any::<[u8; 4]>(), rhs in any::<[u8; 4]>()) {
            let lhs_u32 = u32::from_be_bytes(lhs);
            let rhs_u32 = u32::from_be_bytes(rhs);
            let expected = lhs_u32.wrapping_sub(rhs_u32);
            prop_assert_eq!(wrapping_sub_b::<4>(&lhs, &rhs), expected.to_be_bytes());
        }

        #[test]
        fn wrapping_mul_b_matches_wrapping_arithmetic(lhs in any::<[u8; 32]>(), rhs in any::<[u8; 32]>()) {
            let lhs_u = U::from(lhs);
            let rhs_u = U::from(rhs);
            let expected = lhs_u.wrapping_mul(&rhs_u);
            prop_assert_eq!(wrapping_mul_b::<32>(&lhs, &rhs), expected.0);
        }

        #[test]
        fn const_wrapping_div_agrees_with_wrapping_div_b(
            numerator in any::<[u8; 32]>(),
            denominator in any::<[u8; 32]>().prop_filter("denominator must be non-zero", |d| *d != [0u8; 32])
        ) {
            let numerator_u = U::from(numerator);
            let denominator_u = U::from(denominator);
            prop_assert_eq!(
                const_wrapping_div(&numerator_u, &denominator_u).0,
                wrapping_div_quo_rem_b::<32>(&numerator, &denominator).0
            );
        }

        #[test]
        fn u_predicates_track_zero_and_true(bytes in any::<[u8; 32]>()) {
            let value = U::from(bytes);
            let is_zero = bytes.iter().all(|&b| b == 0);
            prop_assert_eq!(value.is_zero(), is_zero);
            prop_assert_eq!(value.is_some(), !is_zero);
            prop_assert_eq!(value.is_true(), bytes[31] == 1);
        }

        #[test]
        fn test_u_is_zero(x in any::<[u8; 32]>()) {
            let x = U::from(x);
            let ex = U256::from_be_bytes(x.0);
            assert_eq!(ex.is_zero(), x.is_zero());
        }

        #[test]
        fn test_u_div(x in any::<U>(), y in any::<U>()) {
            let ex = U256::from_be_bytes(x.0);
            let ey = U256::from_be_bytes(y.0);
            assert_eq!((ex.wrapping_div(ey)).to_be_bytes(), x.wrapping_div(&y).0);
        }

        #[test]
        fn test_u_mul(x in any::<U>(), y in any::<U>()) {
            let ex = U256::from_be_bytes(x.0);
            let ey = U256::from_be_bytes(y.0);
            assert_eq!((ex.wrapping_mul(ey)).to_be_bytes(), wrapping_mul(&x,  &y).0);
        }

        #[test]
        fn test_u_mod(x in any::<U>(), y in any::<U>()) {
            let ex = U256::from_be_bytes(x.0);
            let ey = U256::from_be_bytes(y.0);
            assert_eq!((ex % ey).to_be_bytes(), (x % y).0);
        }

        #[test]
        fn test_u_add(x in any::<U>(), y in any::<U>()) {
            let ex = U256::from_be_bytes(x.0);
            let ey = U256::from_be_bytes(y.0);
            let e = U::from(ex.wrapping_add(ey).to_be_bytes::<32>());
            assert_eq!(e, x.wrapping_add(&y), "{e} != {}", x + y);
        }

        #[test]
        fn test_u_sub(x in any::<U>(), y in any::<U>()) {
            let ex = U256::from_be_bytes(x.0);
            let ey = U256::from_be_bytes(y.0);
            assert_eq!((ex.wrapping_sub(ey)).to_be_bytes(), x.wrapping_sub(&y).0);
        }

        #[test]
        fn test_u_cmp(x in any::<U>(), y in any::<U>()) {
            let ex = U256::from_be_bytes(x.0);
            let ey = U256::from_be_bytes(y.0);
            assert_eq!(ex.cmp(&ey), x.cmp(&y));
        }

        #[test]
        fn test_u_to_str(x in any::<U>()) {
            assert_eq!(U256::from_be_bytes(x.0).to_string(), x.to_string());
        }

        #[test]
        fn test_u_shl(x in any::<U>(), i in any::<usize>()) {
            let l = U((U256::from_be_bytes(x.0) << i).to_be_bytes::<32>());
            assert_eq!(l, x << i);
        }

        #[test]
        fn test_u_shr(x in any::<U>(), i in any::<usize>()) {
            let l = U((U256::from_be_bytes(x.0) >> i).to_be_bytes::<32>());
            assert_eq!(l, x >> i);
        }

        #[test]
        fn test_trailing_zeros(x in any::<U>()) {
            assert_eq!(U256::from_be_bytes(x.0).trailing_zeros(), x.trailing_zeros());
        }

        #[test]
        fn test_i_is_zero(x in any::<U>()) {
            let ex = I256::from_be_bytes(x.0);
            assert_eq!(ex.is_zero(), x.is_zero());
        }

        #[test]
        fn test_i_div(x in any::<I>(), y in any::<I>()) {
            let ex = I256::from_be_bytes(x.0);
            let ey = I256::from_be_bytes(y.0);
            assert_eq!((ex / ey).to_be_bytes(), (x / y).0);
        }

        #[test]
        fn test_i_mul(x in any::<I>(), y in any::<I>()) {
            let ex = I256::from_be_bytes(x.0);
            let ey = I256::from_be_bytes(y.0);
            assert_eq!((ex.wrapping_mul(ey)).to_be_bytes(), (x * y).0);
        }

        #[test]
        fn test_i_mod(x in any::<I>(), y in any::<I>()) {
            let ex = I256::from_be_bytes(x.0);
            let ey = I256::from_be_bytes(y.0);
            assert_eq!((ex % ey).to_be_bytes(), (x % y).0);
        }

        #[test]
        fn test_i_add(x in any::<I>(), y in any::<I>()) {
            let ex = I256::from_be_bytes(x.0);
            let ey = I256::from_be_bytes(y.0);
            assert_eq!((ex.wrapping_add(ey)).to_be_bytes(), (x + y).0);
        }

        #[test]
        fn test_i_sub(x in any::<I>(), y in any::<I>()) {
            let ex = I256::from_be_bytes(x.0);
            let ey = I256::from_be_bytes(y.0);
            assert_eq!((ex.wrapping_sub(ey)).to_be_bytes(), (x - y).0);
        }

        #[test]
        fn test_i_cmp(x in any::<I>(), y in any::<I>()) {
            let ex = I256::from_be_bytes(x.0);
            let ey = I256::from_be_bytes(y.0);
            assert_eq!(ex.cmp(&ey), x.cmp(&y));
        }

        #[test]
        fn test_u_u8(x in any::<u8>()) {
            let mut b = [0u8; 32];
            b[32-size_of::<u8>()..].copy_from_slice(&x.to_be_bytes());
            assert_eq!(&U256::from_be_bytes(b).to_be_bytes(), U::from(x).as_slice());
        }

        #[test]
        fn test_u_u16(x in any::<u16>()) {
            let mut b = [0u8; 32];
            b[32-size_of::<u16>()..].copy_from_slice(&x.to_be_bytes());
            assert_eq!(&U256::from_be_bytes(b).to_be_bytes(), U::from(x).as_slice());
        }

        #[test]
        fn test_u_u32(x in any::<u32>()) {
            let mut b = [0u8; 32];
            b[32-size_of::<u32>()..].copy_from_slice(&x.to_be_bytes());
            assert_eq!(&U256::from_be_bytes(b).to_be_bytes(), U::from(x).as_slice());
        }

        #[test]
        fn test_u_u64(x in any::<u64>()) {
            let mut b = [0u8; 32];
            b[32-size_of::<u64>()..].copy_from_slice(&x.to_be_bytes());
            assert_eq!(&U256::from_be_bytes(b).to_be_bytes(), U::from(x).as_slice());
        }

        #[test]
        fn test_u_u128(x in any::<u128>()) {
            let mut b = [0u8; 32];
            b[32-size_of::<u128>()..].copy_from_slice(&x.to_be_bytes());
            assert_eq!(&U256::from_be_bytes(b).to_be_bytes(), U::from(x).as_slice());
        }

        #[test]
        fn test_to_and_from_addrs(x in any::<Address>()) {
            let y: Address = U::from(x).into();
            assert_eq!(x, y)
        }

        #[test]
        fn test_u_conv_to_and_from_u8(x in any::<u8>()) {
            assert_eq!(x.wrapping_add(1), U::from(x).wrapping_add(&U::ONE).into());
        }

        #[test]
        fn test_print_to_and_from(x in any::<[u8; 32]>()) {
            let e = format!("{}", U256::from_be_bytes(x));
            let v = format!("{}", U(x));
            assert_eq!(e, v);
        }

        #[test]
        fn test_u_from_str(x in strat_any_u256()) {
            let v = U::from_str(x.to_string().as_str()).unwrap();
            assert_eq!(
                U::from(x.to_be_bytes::<32>()),
                v,
                "{x} != {v}",
            )
        }

        #[test]
        fn array_truncate(x in any::<[u8; 20]>()) {
            assert_eq!(x, U::from(x).const_addr());
        }
    }
}
