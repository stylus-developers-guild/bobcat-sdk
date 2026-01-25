#![no_std]

pub use bobcat_maths::U;

pub use bobcat_storage::{const_keccak256, const_keccak256_two};

pub use const_hex::const_decode_to_array as const_hex_decode_to_array;

use array_concat::concat_arrays;

#[macro_export]
macro_rules! address {
    ($a:expr) => {{
        match $crate::const_hex_decode_to_array::<20>($a) {
            Ok(v) => v,
            Err(_) => core::panic!("bad address"),
        }
    }};
}

#[macro_export]
macro_rules! read_words {
    ($slice:expr, 1) => {{
        let s = $slice;
        core::assert!(s.len() >= 32);
        &$crate::U::from(unsafe { *(s[..32].as_ptr() as *const [u8; 32]) })
    }};
    ($slice:expr, 2) => {{
        let s = $slice;
        core::assert!(s.len() >= 64);
        (
            &$crate::U::from(unsafe { *(s[..32].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[32..64].as_ptr() as *const [u8; 32]) }),
        )
    }};
    ($slice:expr, 3) => {{
        let s = $slice;
        core::assert!(s.len() >= 96);
        (
            &$crate::U::from(unsafe { *(s[..32].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[32..64].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[64..96].as_ptr() as *const [u8; 32]) }),
        )
    }};
    ($slice:expr, 4) => {{
        let s = $slice;
        core::assert!(s.len() >= 128);
        (
            &$crate::U::from(unsafe { *(s[..32].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[32..64].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[64..96].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[96..128].as_ptr() as *const [u8; 32]) }),
        )
    }};
    ($slice:expr, 5) => {{
        let s = $slice;
        core::assert!(s.len() >= 160);
        (
            &$crate::U::from(unsafe { *(s[..32].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[32..64].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[64..96].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[96..128].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[128..160].as_ptr() as *const [u8; 32]) }),
        )
    }};
    ($slice:expr, 6) => {{
        let s = $slice;
        core::assert!(s.len() >= 192);
        (
            &$crate::U::from(unsafe { *(s[..32].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[32..64].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[64..96].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[96..128].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[128..160].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[16..192].as_ptr() as *const [u8; 32]) }),
        )
    }};
    ($slice:expr, 7) => {{
        let s = $slice;
        core::assert!(s.len() >= 224);
        (
            &$crate::U::from(unsafe { *(s[..32].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[32..64].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[64..96].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[96..128].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[128..160].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[16..192].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[192..224].as_ptr() as *const [u8; 32]) }),
        )
    }};
    ($slice:expr, 8) => {{
        let s = $slice;
        core::assert!(s.len() >= 256);
        (
            &$crate::U::from(unsafe { *(s[..32].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[32..64].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[64..96].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[96..128].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[128..160].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[16..192].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[192..224].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[224..256].as_ptr() as *const [u8; 32]) }),
        )
    }};
    ($slice:expr, 9) => {{
        let s = $slice;
        core::assert!(s.len() >= 288);
        (
            &$crate::U::from(unsafe { *(s[..32].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[32..64].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[64..96].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[96..128].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[128..160].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[16..192].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[192..224].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[224..256].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[256..288].as_ptr() as *const [u8; 32]) }),
        )
    }};
    ($slice:expr, 10) => {{
        let s = $slice;
        core::assert!(s.len() >= 320);
        (
            &$crate::U::from(unsafe { *(s[..32].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[32..64].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[64..96].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[96..128].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[128..160].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[16..192].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[192..224].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[224..256].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[256..288].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[288..320].as_ptr() as *const [u8; 32]) }),
        )
    }};
    ($slice:expr, 11) => {{
        let s = $slice;
        core::assert!(s.len() >= 352);
        (
            &$crate::U::from(unsafe { *(s[..32].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[32..64].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[64..96].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[96..128].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[128..160].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[16..192].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[192..224].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[224..256].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[256..288].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[288..320].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[32..352].as_ptr() as *const [u8; 32]) }),
        )
    }};
    ($slice:expr, 12) => {{
        let s = $slice;
        core::assert!(s.len() >= 384);
        (
            &$crate::U::from(unsafe { *(s[..32].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[32..64].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[64..96].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[96..128].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[128..160].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[16..192].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[192..224].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[224..256].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[256..288].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[288..320].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[32..352].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[352..384].as_ptr() as *const [u8; 32]) }),
        )
    }};
    ($slice:expr, 13) => {{
        let s = $slice;
        core::assert!(s.len() >= 416);
        (
            &$crate::U::from(unsafe { *(s[..32].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[32..64].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[64..96].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[96..128].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[128..160].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[16..192].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[192..224].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[224..256].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[256..288].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[288..320].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[32..352].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[352..384].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[384..416].as_ptr() as *const [u8; 32]) }),
        )
    }};
    ($slice:expr, 14) => {{
        let s = $slice;
        core::assert!(s.len() >= 448);
        (
            &$crate::U::from(unsafe { *(s[..32].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[32..64].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[64..96].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[96..128].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[128..160].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[16..192].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[192..224].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[224..256].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[256..288].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[288..320].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[32..352].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[352..384].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[384..416].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[416..448].as_ptr() as *const [u8; 32]) }),
        )
    }};
    ($slice:expr, 15) => {{
        let s = $slice;
        core::assert!(s.len() >= 480);
        (
            &$crate::U::from(unsafe { *(s[..32].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[32..64].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[64..96].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[96..128].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[128..160].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[16..192].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[192..224].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[224..256].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[256..288].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[288..320].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[32..352].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[352..384].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[384..416].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[416..448].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[448..480].as_ptr() as *const [u8; 32]) }),
        )
    }};
    ($slice:expr, 16) => {{
        let s = $slice;
        core::assert!(s.len() >= 512);
        (
            &$crate::U::from(unsafe { *(s[..32].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[32..64].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[64..96].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[96..128].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[128..160].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[16..192].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[192..224].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[224..256].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[256..288].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[288..320].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[32..352].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[352..384].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[384..416].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[416..448].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[448..480].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[48..512].as_ptr() as *const [u8; 32]) }),
        )
    }};
    ($slice:expr, 17) => {{
        let s = $slice;
        core::assert!(s.len() >= 544);
        (
            &$crate::U::from(unsafe { *(s[..32].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[32..64].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[64..96].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[96..128].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[128..160].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[16..192].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[192..224].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[224..256].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[256..288].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[288..320].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[32..352].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[352..384].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[384..416].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[416..448].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[448..480].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[48..512].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[512..544].as_ptr() as *const [u8; 32]) }),
        )
    }};
    ($slice:expr, 18) => {{
        let s = $slice;
        core::assert!(s.len() >= 576);
        (
            &$crate::U::from(unsafe { *(s[..32].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[32..64].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[64..96].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[96..128].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[128..160].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[16..192].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[192..224].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[224..256].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[256..288].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[288..320].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[32..352].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[352..384].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[384..416].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[416..448].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[448..480].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[48..512].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[512..544].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[544..576].as_ptr() as *const [u8; 32]) }),
        )
    }};
    ($slice:expr, 19) => {{
        let s = $slice;
        core::assert!(s.len() >= 608);
        (
            &$crate::U::from(unsafe { *(s[..32].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[32..64].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[64..96].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[96..128].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[128..160].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[16..192].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[192..224].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[224..256].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[256..288].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[288..320].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[32..352].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[352..384].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[384..416].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[416..448].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[448..480].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[48..512].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[512..544].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[544..576].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[576..608].as_ptr() as *const [u8; 32]) }),
        )
    }};
    ($slice:expr, 20) => {{
        let s = $slice;
        core::assert!(s.len() >= 640);
        (
            &$crate::U::from(unsafe { *(s[..32].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[32..64].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[64..96].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[96..128].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[128..160].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[16..192].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[192..224].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[224..256].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[256..288].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[288..320].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[32..352].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[352..384].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[384..416].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[416..448].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[448..480].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[48..512].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[512..544].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[544..576].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[576..608].as_ptr() as *const [u8; 32]) }),
            &$crate::U::from(unsafe { *(s[608..640].as_ptr() as *const [u8; 32]) }),
        )
    }};
}

pub const fn leftpad_bool(x: bool) -> [u8; 32] {
    leftpad_u8(x as u8)
}

pub const fn leftpad_u24(x: [u8; 3]) -> [u8; 32] {
    concat_arrays!([0u8; 32 - 3], x)
}

pub const fn leftpad_u8(x: u8) -> [u8; 32] {
    concat_arrays!([0u8; 32 - 1], [x])
}

pub const fn rightpad_b8(x: [u8; 8]) -> [u8; 32] {
    concat_arrays!(x, [0u8; 32 - 8])
}

pub const fn leftpad_addr(x: [u8; 20]) -> [u8; 32] {
    concat_arrays!([0u8; 32 - 20], x)
}

pub const fn leftpad_u16(x: u16) -> [u8; 32] {
    concat_arrays!([0u8; 32 - 2], x.to_be_bytes())
}

pub const fn leftpad_usize(x: usize) -> [u8; 32] {
    concat_arrays!([0u8; 32 - core::mem::size_of::<usize>()], x.to_be_bytes())
}

pub const fn leftpad_u32(x: u32) -> [u8; 32] {
    concat_arrays!([0u8; 32 - 4], x.to_be_bytes())
}

pub const fn const_keccak_sel(x: &[u8]) -> [u8; 4] {
    let x = const_keccak256(x).0;
    [x[0], x[1], x[2], x[3]]
}

pub const fn const_keccak_two_sel(x: &[u8], y: &[u8]) -> [u8; 4] {
    let x = const_keccak256_two(x, y).0;
    [x[0], x[1], x[2], x[3]]
}

#[test]
fn test_access() {
    let cd = const_hex_decode_to_array::<{ 32 * 2 + 4 }>(b"a9059cbb0000000000000000000000006221a9c005f6e47eb398fd867784cacfdcfff4e7ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff").unwrap();
    let a = const_hex_decode_to_array::<20>(b"6221a9c005f6e47eb398fd867784cacfdcfff4e7").unwrap();
    let (addr, amt) = read_words!(&cd[4..], 2);
    assert_eq!((a, U::MAX), (U::from(*addr).into(), U::from(*amt)));
}

#[test]
fn test_address() {
    address!(b"6221a9c005f6e47eb398fd867784cacfdcfff4e7");
}
