#![no_std]

pub use bobcat_storage::{const_keccak256_two_off_curve, storage_load, storage_store, U};

#[macro_export]
macro_rules! bobcat_feature {
    ($feature_name:ident) => {
        paste::paste! {
            #[allow(unused)]
            macro_rules! [<IF_FEATURE_ $feature_name:upper>] {
                ($on_block:block, $off_block:block) => {
                    if $crate::storage_load(&$crate::const_keccak256_two_off_curve(
                        b"bobcat.features.",
                        stringify!($feature_name).as_bytes()
                    )).is_some() {
                        $on_block
                    } else {
                        $off_block
                    }
                };
            }

            #[allow(unused)]
            macro_rules! [<FEATURE_SET_ $feature_name:upper>] {
                ($value:expr) => {
                    $crate::storage_store(
                        &$crate::const_keccak256_two_off_curve(
                            b"bobcat.features.",
                            stringify!($feature_name).as_bytes()
                        ),
                        &$crate::U::from($value)
                    )
                };
            }
        }
    };
}

#[cfg(all(test, feature = "std"))]
mod test {
    bobcat_feature!(test123);

    #[test]
    fn test_feature() {
        FEATURE_SET_TEST123!(true);
        assert!(IF_FEATURE_TEST123!({ true }, { false }));
    }
}
