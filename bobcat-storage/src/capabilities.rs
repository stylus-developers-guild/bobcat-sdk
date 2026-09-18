
use core::marker::PhantomData;

pub trait Cap {
    fn write() -> bool;
    fn read() -> bool;
}

macro_rules! capabilities {
    ($t:ident, $cw:expr, $cr:expr) => {
        pub struct $t;

        impl Cap for $t {
            fn write() -> bool { $cw }
            fn read() -> bool { $cr }
        }
    }
}

capabilities! {CW, true, false}
capabilities! {CA, true, true}
capabilities! {CR, false, true}

pub struct EvmBox<C: Cap, T>{
    _phantom: PhantomData<(C, T)>
}
