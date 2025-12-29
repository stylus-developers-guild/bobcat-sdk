#![no_std]

pub use bobcat_maths::U;

#[cfg(not(feature = "shadow"))]
use array_concat::concat_arrays;

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

#[cfg(all(target_family = "wasm", target_os = "unknown"))]
mod impls {
    #[link(wasm_import_module = "vm_hooks")]
    #[allow(unused)]
    unsafe extern "C" {
        pub(crate) fn emit_log(data: *const u8, len: usize, topics: usize);
        pub(crate) fn write_result(ptr: *const u8, len: usize);
    }
}

#[cfg(not(all(target_family = "wasm", target_os = "unknown")))]
#[allow(unused)]
mod impls {
    pub(crate) unsafe fn emit_log(_: *const u8, _: usize, _: usize) {}
    pub(crate) unsafe fn write_result(_: *const u8, _: usize) {}
}

// ShadowTable that's intended for the 32 wasm machine to do shadow event
// logging with custom Nitro instances with a whitelisted pool of nodes
// to log from.
#[derive(Debug, Clone, PartialEq)]
#[repr(C)]
struct ShadowTable<const DATA: usize> {
    magic: u8,
    topics_len: usize,
    data_len: usize,
    topics: [U; 4],
    data: [u8; DATA],
}

#[cfg(feature = "alloc")]
#[derive(Debug, Clone, PartialEq)]
#[repr(C)]
struct ShadowTableVec {
    magic: u8,
    topics_len: usize,
    data_len: usize,
    topics: [U; 4],
    data: Vec<u8>,
}

const SHADOW_MAGIC_BYTE: u8 = 0xee;

impl<const DATA: usize> Default for ShadowTable<DATA> {
    fn default() -> Self {
        Self {
            magic: SHADOW_MAGIC_BYTE,
            topics_len: 0,
            data_len: 0,
            topics: [U::ZERO; 4],
            data: [0u8; DATA],
        }
    }
}

#[cfg(feature = "alloc")]
impl Default for ShadowTableVec {
    fn default() -> Self {
        Self {
            magic: SHADOW_MAGIC_BYTE,
            topics_len: 0,
            data_len: 0,
            topics: [U::ZERO; 4],
            data: Vec::new(),
        }
    }
}

// Shadow functions
pub fn shadow_log_0_slice<const D: usize, const ALL: usize>(t0: &U, d: [u8; D]) {
    let mut t = ShadowTable {
        data_len: D,
        data: d,
        ..ShadowTable::<D>::default()
    };
    t.topics[0] = *t0;
    unsafe {
        impls::write_result(&t as *const _ as *const u8, 0);
    }
}

pub fn shadow_log_1_slice<const D: usize, const ALL: usize>(t0: &U, t1: &U, d: [u8; D]) {
    let mut t = ShadowTable {
        topics_len: 1,
        data_len: D,
        data: d,
        ..ShadowTable::<D>::default()
    };
    t.topics[0] = *t0;
    t.topics[1] = *t1;
    unsafe {
        impls::write_result(&t as *const _ as *const u8, 0);
    }
}

pub fn shadow_log_2_slice<const D: usize, const ALL: usize>(t0: &U, t1: &U, t2: &U, d: [u8; D]) {
    let mut t = ShadowTable {
        topics_len: 2,
        data_len: D,
        data: d,
        ..ShadowTable::<D>::default()
    };
    t.topics[0] = *t0;
    t.topics[1] = *t1;
    t.topics[2] = *t2;
    unsafe {
        impls::write_result(&t as *const _ as *const u8, 0);
    }
}

pub fn shadow_log_3_slice<const D: usize, const ALL: usize>(
    t0: &U,
    t1: &U,
    t2: &U,
    t3: &U,
    d: [u8; D],
) {
    let mut t = ShadowTable {
        topics_len: 3,
        data_len: D,
        data: d,
        ..ShadowTable::<D>::default()
    };
    t.topics[0] = *t0;
    t.topics[1] = *t1;
    t.topics[2] = *t2;
    t.topics[3] = *t3;
    unsafe {
        impls::write_result(&t as *const _ as *const u8, 0);
    }
}

#[cfg(feature = "alloc")]
pub fn shadow_log_count(topics: [U; 4], topics_len: usize, data: &[u8]) {
    let t = ShadowTableVec {
        topics,
        topics_len,
        data_len: data.len(),
        data: data.to_vec(),
        ..ShadowTableVec::default()
    };
    unsafe {
        impls::write_result(&t as *const _ as *const u8, 0);
    }
}

// Emit functions (no feature flags)
#[cfg(not(feature = "shadow"))]
pub fn emit_log_0_slice<const D: usize, const ALL: usize>(t0: &U, d: [u8; D]) {
    assert_eq!(ALL, D + 32, "not properly sized: {}", D + 32);
    let x: [u8; ALL] = concat_arrays!(t0.0, d);
    unsafe {
        impls::emit_log(x.as_ptr(), x.len(), 1);
    }
}

#[cfg(feature = "shadow")]
pub fn emit_log_0_slice<const D: usize, const ALL: usize>(t0: &U, d: [u8; D]) {
    shadow_log_0_slice::<D, ALL>(t0, d);
}

#[cfg(not(feature = "shadow"))]
pub fn emit_log_1_slice<const D: usize, const ALL: usize>(t0: &U, t1: &U, d: [u8; D]) {
    assert_eq!(ALL, D + 32 * 2, "not properly sized: {}", D + 64);
    let x: [u8; ALL] = concat_arrays!(t0.0, t1.0, d);
    unsafe {
        impls::emit_log(x.as_ptr(), x.len(), 2);
    }
}

#[cfg(feature = "shadow")]
pub fn emit_log_1_slice<const D: usize, const ALL: usize>(t0: &U, t1: &U, d: [u8; D]) {
    shadow_log_1_slice::<D, ALL>(t0, t1, d);
}

#[cfg(not(feature = "shadow"))]
pub fn emit_log_2_slice<const D: usize, const ALL: usize>(t0: &U, t1: &U, t2: &U, d: [u8; D]) {
    assert_eq!(ALL, D + 32 * 3, "not properly sized: {}", D + 96);
    let x: [u8; ALL] = concat_arrays!(t0.0, t1.0, t2.0, d);
    unsafe {
        impls::emit_log(x.as_ptr(), x.len(), 3);
    }
}

#[cfg(feature = "shadow")]
pub fn emit_log_2_slice<const D: usize, const ALL: usize>(t0: &U, t1: &U, t2: &U, d: [u8; D]) {
    shadow_log_2_slice::<D, ALL>(t0, t1, t2, d);
}

#[cfg(not(feature = "shadow"))]
pub fn emit_log_3_slice<const D: usize, const ALL: usize>(
    t0: &U,
    t1: &U,
    t2: &U,
    t3: &U,
    d: [u8; D],
) {
    assert_eq!(ALL, D + 32 * 4, "not properly sized: {}", D + 128);
    let x: [u8; ALL] = concat_arrays!(t0.0, t1.0, t2.0, t3.0, d);
    unsafe {
        impls::emit_log(x.as_ptr(), x.len(), 4);
    }
}

#[cfg(feature = "shadow")]
pub fn emit_log_3_slice<const D: usize, const ALL: usize>(
    t0: &U,
    t1: &U,
    t2: &U,
    t3: &U,
    d: [u8; D],
) {
    shadow_log_3_slice::<D, ALL>(t0, t1, t2, t3, d);
}

/// Emit a log with a variable amount of topics, with the topics provided
/// by a slice with a fixed structure. Useful for glue to leverage the
/// shadow feature with stylus-sdk.
#[cfg(all(feature = "alloc", not(feature = "shadow")))]
pub fn emit_log_count(topics: [U; 4], topics_len: usize, data: &[u8]) {
    let mut d = Vec::with_capacity(topics_len * 32 + data.len());
    for t in &topics[..topics_len] {
        d.extend_from_slice(t.as_slice());
    }
    d.extend_from_slice(data);
    unsafe {
        impls::emit_log(d.as_ptr(), data.len(), topics_len);
    }
}

#[cfg(all(feature = "alloc", feature = "shadow"))]
pub fn emit_log_count(topics: [U; 4], topics_len: usize, data: &[u8]) {
    shadow_log_count(topics, topics_len, data);
}

#[cfg(feature = "alloc")]
pub fn emit_log_0_vec(t0: &U, d: &[u8]) {
    let mut x = t0.to_vec();
    x.extend_from_slice(d);
    unsafe {
        impls::emit_log(x.as_ptr(), x.len(), 1);
    }
}

#[cfg(feature = "alloc")]
pub fn emit_log_1_vec(t0: &U, t1: &U, d: &[u8]) {
    let mut x = t0.to_vec();
    x.extend_from_slice(&t1.0);
    x.extend_from_slice(d);
    unsafe {
        impls::emit_log(x.as_ptr(), x.len(), 2);
    }
}

#[cfg(feature = "alloc")]
pub fn emit_log_2_vec(t0: &U, t1: &U, t2: &U, d: &[u8]) {
    let mut x = t0.to_vec();
    x.extend_from_slice(&t1.0);
    x.extend_from_slice(&t2.0);
    x.extend_from_slice(d);
    unsafe {
        impls::emit_log(x.as_ptr(), x.len(), 3);
    }
}

#[cfg(feature = "alloc")]
pub fn emit_log_3_vec(t0: &U, t1: &U, t2: &U, t3: &U, d: &[u8]) {
    let mut x = t0.to_vec();
    x.extend_from_slice(&t1.0);
    x.extend_from_slice(&t2.0);
    x.extend_from_slice(&t3.0);
    x.extend_from_slice(d);
    unsafe {
        impls::emit_log(x.as_ptr(), x.len(), 4);
    }
}

// Emit an event, doing some into() use and copying of reference Us to
// make it easier to do printing.
#[macro_export]
macro_rules! emit {
    ($t0:expr) => {{
        const DATA_LEN: usize = 0;
        const ALL_LEN: usize = 32;
        let t0: $crate::U = $t0.into();
        $crate::emit_log_0_slice::<DATA_LEN, ALL_LEN>(&t0, [])
    }};

    ($t0:expr, data: $data:expr) => {{
        let t0: $crate::U = $t0.into();
        $crate::emit_log_0_vec(&t0, $data)
    }};

    ($t0:expr, data: $data:expr, $data_len:expr) => {{
        const DATA_LEN: usize = $data_len;
        const ALL_LEN: usize = DATA_LEN + 32;
        let t0: $crate::U = $t0.into();
        $crate::emit_log_0_slice::<DATA_LEN, ALL_LEN>(&t0, $data)
    }};

    ($t0:expr, $t1:expr) => {{
        const DATA_LEN: usize = 0;
        const ALL_LEN: usize = 32 * 2;
        let t0: $crate::U = $t0.into();
        let t1: $crate::U = $t1.into();
        $crate::emit_log_1_slice::<DATA_LEN, ALL_LEN>(&t0, &t1, [])
    }};

    ($t0:expr, $t1:expr, data: $data:expr) => {{
        let t0: $crate::U = $t0.into();
        let t1: $crate::U = $t1.into();
        $crate::emit_log_1_vec(&t0, &t1, $data)
    }};

    ($t0:expr, $t1:expr, data: $data:expr, $data_len:expr) => {{
        const DATA_LEN: usize = $data_len;
        const ALL_LEN: usize = DATA_LEN + 32 * 2;
        let t0: $crate::U = $t0.into();
        let t1: $crate::U = $t1.into();
        $crate::emit_log_1_slice::<DATA_LEN, ALL_LEN>(&t0, &t1, $data)
    }};

    ($t0:expr, $t1:expr, $t2:expr) => {{
        const DATA_LEN: usize = 0;
        const ALL_LEN: usize = 32 * 3;
        let t0: $crate::U = $t0.into();
        let t1: $crate::U = $t1.into();
        let t2: $crate::U = $t2.into();
        $crate::emit_log_2_slice::<DATA_LEN, ALL_LEN>(&t0, &t1, &t2, [])
    }};

    ($t0:expr, $t1:expr, $t2:expr, data: $data:expr) => {{
        let t0: $crate::U = $t0.into();
        let t1: $crate::U = $t1.into();
        let t2: $crate::U = $t2.into();
        $crate::emit_log_2_vec(&t0, &t1, &t2, $data)
    }};

    ($t0:expr, $t1:expr, $t2:expr, data: $data:expr, $data_len:expr) => {{
        const DATA_LEN: usize = $data_len;
        const ALL_LEN: usize = DATA_LEN + 32 * 3;
        let t0: $crate::U = $t0.into();
        let t1: $crate::U = $t1.into();
        let t2: $crate::U = $t2.into();
        $crate::emit_log_2_slice::<DATA_LEN, ALL_LEN>(&t0, &t1, &t2, $data)
    }};

    ($t0:expr, $t1:expr, $t2:expr, $t3:expr) => {{
        const DATA_LEN: usize = 0;
        const ALL_LEN: usize = 32 * 4;
        let t0: $crate::U = $t0.into();
        let t1: $crate::U = $t1.into();
        let t2: $crate::U = $t2.into();
        let t3: $crate::U = $t3.into();
        $crate::emit_log_3_slice::<DATA_LEN, ALL_LEN>(&t0, &t1, &t2, &t3, [])
    }};

    ($t0:expr, $t1:expr, $t2:expr, $t3:expr, data: $data:expr) => {{
        let t0: $crate::U = $t0.into();
        let t1: $crate::U = $t1.into();
        let t2: $crate::U = $t2.into();
        let t3: $crate::U = $t3.into();
        $crate::emit_log_3_vec(&t0, &t1, &t2, &t3, $data)
    }};

    ($t0:expr, $t1:expr, $t2:expr, $t3:expr, data: $data:expr, $data_len:expr) => {{
        const DATA_LEN: usize = $data_len;
        const ALL_LEN: usize = DATA_LEN + 32 * 4;
        let t0: $crate::U = $t0.into();
        let t1: $crate::U = $t1.into();
        let t2: $crate::U = $t2.into();
        let t3: $crate::U = $t3.into();
        $crate::emit_log_3_slice::<DATA_LEN, ALL_LEN>(&t0, &t1, &t2, &t3, $data)
    }};
}

#[macro_export]
macro_rules! shadow {
    ($t0:expr) => {{
        const DATA_LEN: usize = 0;
        const ALL_LEN: usize = 32;
        let t0: $crate::U = $t0.into();
        $crate::shadow_log_0_slice::<DATA_LEN, ALL_LEN>(&t0, [])
    }};

    ($t0:expr, data: $data:expr) => {{
        let t0: $crate::U = $t0.into();
        $crate::shadow_log_0_vec(&t0, $data)
    }};

    ($t0:expr, data: $data:expr, $data_len:expr) => {{
        const DATA_LEN: usize = $data_len;
        const ALL_LEN: usize = DATA_LEN + 32;
        let t0: $crate::U = $t0.into();
        $crate::shadow_log_0_slice::<DATA_LEN, ALL_LEN>(&t0, $data)
    }};

    ($t0:expr, $t1:expr) => {{
        const DATA_LEN: usize = 0;
        const ALL_LEN: usize = 32 * 2;
        let t0: $crate::U = $t0.into();
        let t1: $crate::U = $t1.into();
        $crate::shadow_log_1_slice::<DATA_LEN, ALL_LEN>(&t0, &t1, [])
    }};

    ($t0:expr, $t1:expr, data: $data:expr) => {{
        let t0: $crate::U = $t0.into();
        let t1: $crate::U = $t1.into();
        $crate::shadow_log_1_vec(&t0, &t1, $data)
    }};

    ($t0:expr, $t1:expr, data: $data:expr, $data_len:expr) => {{
        const DATA_LEN: usize = $data_len;
        const ALL_LEN: usize = DATA_LEN + 32 * 2;
        let t0: $crate::U = $t0.into();
        let t1: $crate::U = $t1.into();
        $crate::shadow_log_1_slice::<DATA_LEN, ALL_LEN>(&t0, &t1, $data)
    }};

    ($t0:expr, $t1:expr, $t2:expr) => {{
        const DATA_LEN: usize = 0;
        const ALL_LEN: usize = 32 * 3;
        let t0: $crate::U = $t0.into();
        let t1: $crate::U = $t1.into();
        let t2: $crate::U = $t2.into();
        $crate::shadow_log_2_slice::<DATA_LEN, ALL_LEN>(&t0, &t1, &t2, [])
    }};

    ($t0:expr, $t1:expr, $t2:expr, data: $data:expr) => {{
        let t0: $crate::U = $t0.into();
        let t1: $crate::U = $t1.into();
        let t2: $crate::U = $t2.into();
        $crate::shadow_log_2_vec(&t0, &t1, &t2, $data)
    }};

    ($t0:expr, $t1:expr, $t2:expr, data: $data:expr, $data_len:expr) => {{
        const DATA_LEN: usize = $data_len;
        const ALL_LEN: usize = DATA_LEN + 32 * 3;
        let t0: $crate::U = $t0.into();
        let t1: $crate::U = $t1.into();
        let t2: $crate::U = $t2.into();
        $crate::shadow_log_2_slice::<DATA_LEN, ALL_LEN>(&t0, &t1, &t2, $data)
    }};

    ($t0:expr, $t1:expr, $t2:expr, $t3:expr) => {{
        const DATA_LEN: usize = 0;
        const ALL_LEN: usize = 32 * 4;
        let t0: $crate::U = $t0.into();
        let t1: $crate::U = $t1.into();
        let t2: $crate::U = $t2.into();
        let t3: $crate::U = $t3.into();
        $crate::shadow_log_3_slice::<DATA_LEN, ALL_LEN>(&t0, &t1, &t2, &t3, [])
    }};

    ($t0:expr, $t1:expr, $t2:expr, $t3:expr, data: $data:expr) => {{
        let t0: $crate::U = $t0.into();
        let t1: $crate::U = $t1.into();
        let t2: $crate::U = $t2.into();
        let t3: $crate::U = $t3.into();
        $crate::shadow_log_3_vec(&t0, &t1, &t2, &t3, $data)
    }};

    ($t0:expr, $t1:expr, $t2:expr, $t3:expr, data: $data:expr, $data_len:expr) => {{
        const DATA_LEN: usize = $data_len;
        const ALL_LEN: usize = DATA_LEN + 32 * 4;
        let t0: $crate::U = $t0.into();
        let t1: $crate::U = $t1.into();
        let t2: $crate::U = $t2.into();
        let t3: $crate::U = $t3.into();
        $crate::shadow_log_3_slice::<DATA_LEN, ALL_LEN>(&t0, &t1, &t2, &t3, $data)
    }};
}
