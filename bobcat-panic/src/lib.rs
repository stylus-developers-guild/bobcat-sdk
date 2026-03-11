#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

#[allow(unused)]
use core::fmt::{Result as FmtResult, Write};

use bobcat_host as host;

use paste::paste;

//Panic(uint256)
pub const PANIC_PREAMBLE_WORD: [u8; 32 + 4] = match const_hex::const_decode_to_array::<{ 32 + 4 }>(
    b"4e487b710000000000000000000000000000000000000000000000000000000000000000",
) {
    Ok(v) => v,
    Err(_) => panic!(),
};

//Error(string)
pub const ERROR_PREAMBLE_OFFSET: [u8; 4 + 32] = match const_hex::const_decode_to_array::<{ 4 + 32 }>(
    b"08c379a00000000000000000000000000000000000000000000000000000000000000020",
) {
    Ok(v) => v,
    Err(_) => panic!(),
};

#[derive(Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum PanicCodes {
    DecodingError = 0,
    OverflowOrUnderflow = 0x11,
    DivByZero = 0x12,
}

#[cfg(all(target_family = "wasm", target_os = "unknown"))]
pub fn panic_with_code(x: PanicCodes) -> ! {
    let mut b = PANIC_PREAMBLE_WORD;
    b[4 + 32 - 1] = x as u8;
    unsafe {
        host::write_result(b.as_ptr(), b.len());
        host::exit_early(1)
    }
}

#[cfg(not(all(target_family = "wasm", target_os = "unknown")))]
pub fn panic_with_code(x: PanicCodes) -> ! {
    panic!("panicked with code: {x:?}");
}

#[macro_export]
macro_rules! define_panic_macros {
    (@internal [$dollar:tt] $(($error_msg:expr, $panic_code:ident)),* $(,)?) => {
        $(
            paste! {
                #[macro_export]
                macro_rules! [<panic_on_err_ $error_msg>] {
                    ($dollar e:expr; $dollar($dollar arg:tt)*) => {{
                        match $dollar e {
                            Some(v) => v,
                            None => {
                                #[cfg(feature = "detailed-errors")]
                                panic!($dollar($dollar arg)*);
                                #[cfg(all(not(feature = "detailed-errors"), feature = "msg-on-sdk-err"))]
                                panic!("internal sdk (overflow/div by zero?)");
                                #[cfg(feature = "panic-code")]
                                $crate::panic_with_code($crate::PanicCodes::$panic_code);
                                #[cfg(not(any(feature = "detailed-errors", feature = "msg-on-sdk-err", feature = "panic-code")))]
                                panic!()
                            }
                        }
                    }};
                }
            }
        )*
    };
    ($(($error_msg:expr, $panic_code:ident)),* $(,)?) => {
        $crate::define_panic_macros!(@internal [$] $(($error_msg, $panic_code)),*);
    };
}

define_panic_macros!(
    ("overflow", OverflowOrUnderflow),
    ("div_by_zero", DivByZero),
);

#[macro_export]
macro_rules! panic_on_err_bad_decoding_bool {
    ($e:expr; $($arg:tt)*) => {{
        if !$e {
            $crate::panic_on_err_bad_decoding_bool!($($arg)*);
        }
    }};
    ($($arg:tt)*) => {{
        #[cfg(feature = "detailed-errors")]
        panic!($($arg)*);
        #[cfg(all(not(feature = "detailed-errors"), feature = "msg-on-sdk-err"))]
        panic!("decoding error");
        #[cfg(feature = "panic-code")]
        $crate::panic_with_code($crate::PanicCodes::DecodingError);
        #[cfg(not(any(feature = "detailed-errors", feature = "msg-on-sdk-err", feature = "panic-code")))]
        panic!()
    }};
}

#[allow(unused)]
struct SliceWriter<'a>(&'a mut [u8], usize);

impl<'a> Write for SliceWriter<'a> {
    fn write_str(&mut self, s: &str) -> FmtResult {
        let v = s.len().min(REVERT_BUF_SIZE.saturating_sub(self.1));
        self.0[self.1..self.1 + v].copy_from_slice(&s.as_bytes()[..v]);
        self.1 += v;
        Ok(())
    }
}

//uint256(keccak256(abi.encodePacked("bobcat.tracing.counter"))) - 1
pub const SLOT_TRACING_COUNTER: [u8; 32] = [
    0xad, 0x59, 0xcd, 0x5c, 0xcd, 0xcd, 0x00, 0x59, 0x2c, 0xd2, 0x06, 0xdc, 0x3b, 0xce, 0x83, 0xac,
    0xe6, 0x1b, 0x8c, 0x80, 0xcb, 0xe9, 0xfd, 0x0d, 0x70, 0x09, 0x34, 0xba, 0x13, 0x78, 0x92, 0x22,
];

/// Revert buffer size that's used to write the panic. We can afford to
/// use a large page here since a panic will consume all the gas anyway,
/// and a user will see this during simulation hopefully.
#[allow(unused)]
const REVERT_BUF_SIZE: usize = 1024 * 10;

#[cfg(all(feature = "panic-revert", feature = "panic-loc"))]
compile_error!("panic-revert and panic-loc simultaneously enabled");

#[cfg(all(feature = "panic-revert", feature = "panic-trace"))]
compile_error!("panic-revert and panic-trace simultaneously enabled");

#[cfg(all(feature = "panic-loc", feature = "panic-trace"))]
compile_error!("panic-loc and panic-trace simultaneously enabled");

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
enum TracingDiscriminant {
    Number = 0,
    String = 1,
}

#[cfg(all(target_family = "wasm", target_os = "unknown"))]
#[cfg_attr(all(feature = "panic", not(feature = "std")), panic_handler)]
pub fn panic_handler(_msg: &core::panic::PanicInfo) -> ! {
    #[cfg(feature = "console")]
    {
        let msg = alloc::format!("{_msg}");
        unsafe { host::log_txt(msg.as_ptr(), msg.len()) }
    }
    #[cfg(any(
        feature = "panic-revert",
        feature = "panic-loc",
        feature = "panic-trace"
    ))]
    {
        let mut buf = [0u8; REVERT_BUF_SIZE];
        buf[..ERROR_PREAMBLE_OFFSET.len()].copy_from_slice(&ERROR_PREAMBLE_OFFSET);
        let mut w = SliceWriter(&mut buf[ERROR_PREAMBLE_OFFSET.len() + 32..], 0);
        #[cfg(feature = "panic-revert")]
        {
            write!(&mut w, "{_msg}").unwrap();
        }
        #[cfg(feature = "panic-loc")]
        if let Some(loc) = _msg.location() {
            write!(&mut w, "panic: {}:{}", loc.file(), loc.line()).unwrap();
        } else {
            write!(&mut w, "panic: unknown").unwrap();
        }
        #[cfg(feature = "panic-trace")]
        {
            let mut b = [0u8; 32];
            unsafe { host::transient_load_bytes32(SLOT_TRACING_COUNTER.as_ptr(), b.as_mut_ptr()) };
            if b[0] == TracingDiscriminant::Number as u8 {
                write!(
                    &mut w,
                    "trace no: {}",
                    u32::from_be_bytes(b[1..32 - size_of::<u32>()].try_into().unwrap())
                )
            } else {
                write!(&mut w, "trace str: {}", trace_key_to_str(&b))
            }
            .unwrap()
        }
        let len_msg = w.1;
        let len_offset = ERROR_PREAMBLE_OFFSET.len();
        buf[len_offset + 28..len_offset + 32].copy_from_slice(&(len_msg as u32).to_be_bytes());
        let len_full = ERROR_PREAMBLE_OFFSET.len() + 32 + len_msg;
        let len_padded = len_full + (32 - (len_full % 32)) % 32;
        unsafe {
            host::write_result(buf.as_ptr(), len_padded);
            host::exit_early(1)
        }
    }
    // Prefer the normal behaviour if the user hasn't opted into this
    // feature. Maybe it's better to wipe out the revertdata if this happens,
    // the other behaviour is different.
    #[allow(unreachable_code)]
    core::arch::wasm32::unreachable()
}

#[cfg(all(target_arch = "riscv32", target_os = "none"))]
#[cfg_attr(all(feature = "panic", not(feature = "std")), panic_handler)]
pub fn panic_handler(_: &core::panic::PanicInfo) -> ! {
    // TODO: this needs to be fleshed out
    unsafe {
        core::arch::asm!("ebreak");
    }
    loop {}
}

pub fn bump() {
    let p = SLOT_TRACING_COUNTER.as_ptr();
    // We assume the execution counter here is always less than u32,
    // so the upper part of the word could be dirty!
    let mut b = [0u8; 32];
    unsafe { host::transient_load_bytes32(p, b.as_mut_ptr()) };
    let v = u32::from_be_bytes(b[32 - size_of::<u32>()..].try_into().unwrap()) + 1;
    b[32 - size_of::<u32>()..].copy_from_slice(&v.to_be_bytes());
    b[0] = TracingDiscriminant::Number as u8;
    unsafe { host::transient_store_bytes32(p, b.as_ptr()) }
}

pub const fn trace_key_of_str(s: &str) -> [u8; 32] {
    let bytes = s.as_bytes();
    let mut b = [0u8; 32];
    b[0] = TracingDiscriminant::String as u8;
    let mut i = 0;
    while i < bytes.len() && i < 31 {
        b[i + 1] = bytes[i];
        i += 1;
    }
    b
}

#[allow(unused)]
const fn trace_key_to_str(b: &[u8; 32]) -> &str {
    let mut i = 1;
    while i < 32 {
        if b[i] == 0 {
            break;
        }
        i += 1;
    }
    unsafe {
        let slice = core::slice::from_raw_parts(b.as_ptr().add(1), i - 1);
        core::str::from_utf8_unchecked(slice)
    }
}

pub fn trace(k: &str) {
    let v = trace_key_of_str(k);
    unsafe { host::transient_store_bytes32(SLOT_TRACING_COUNTER.as_ptr(), v.as_ptr()) }
}

#[macro_export]
macro_rules! trace_guard {
    ($($body:tt)*) => {{
        trace(concat!(file!(), ":", line!()));
        $($body)*
    }};
}

#[cfg(all(test, feature = "proptest"))]
mod test {
    use proptest::prelude::*;

    use super::*;

    proptest! {
        #[test]
        fn test_key_back_and_forth(x in proptest::string::string_regex("[0-9a-zA-Z]{0,31}").unwrap()) {
            assert_eq!(&x, trace_key_to_str(&trace_key_of_str(&x)));
        }
    }
}
