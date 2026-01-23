#![no_std]

extern crate alloc;

pub use alloc::format;

pub use bobcat_host::log_txt;

#[macro_export]
macro_rules! console {
    ($val:expr) => {
    {
        let tmp = $val;
        let msg = $crate::format!("[{}:{}] {} = {:#?}\n", file!(), line!(), stringify!($val), &tmp);
        unsafe { $crate::log_txt(msg.as_ptr(), msg.len()) };
        tmp
    }};
    ($($vals:expr),+ $(,)?) => {
    {
        let tup = ($($vals),+);
        let msg = $crate::format!("[{}:{}] {} = {:#?}\n", file!(), line!(), stringify!(($($vals),+)), &tup);
        unsafe { $crate::log_txt(msg.as_ptr(), msg.len()) };
        tup
    }};
}
