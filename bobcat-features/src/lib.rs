#![cfg_attr(not(feature = "std"), no_std)]

pub use bobcat_storage::{
    const_keccak256_two_off_curve, keccak256, storage_load, storage_store, U,
};

pub use bobcat_entry::block_timestamp;

#[macro_export]
macro_rules! bobcat_features {
    ($($feature_name:ident),* $(,)?) => {
        pub const _FEATURE_COUNT: usize = 0 $(+ { let _ = stringify!($feature_name); 1 })*;

        $(
            paste::paste! {
                pub const [<FEATURE_ID_ $feature_name:upper>]: $crate::U =
                    $crate::const_keccak256_two_off_curve(
                        b"bobcat.features.",
                        stringify!($feature_name).as_bytes()
                    );

                #[allow(unused)]
                pub fn [<feature_is_ $feature_name:lower>]() -> bool {
                    $crate::storage_load(&[<FEATURE_ID_ $feature_name:upper>]).is_some()
                }

                pub fn [<feature_set_ $feature_name:lower>](v: bool) {
                    $crate::storage_store(
                        &[<FEATURE_ID_ $feature_name:upper>],
                        &$crate::U::from(v)
                    )
                }

                #[allow(unused)]
                macro_rules! [<FEATURE_IF_ $feature_name:upper>] {
                    ($on_block:block else $off_block:block) => {
                        if [<feature_is_ $feature_name:lower>]() {
                            $on_block
                        } else {
                            $off_block
                        }
                    };
                }
            }
        )*
    };
}

#[macro_export]
macro_rules! FEATURE_PICK {
    ($($feature_name:ident($weight:expr)),* $(,)?) => {
        {
            let choice = u16::from($crate::keccak256(&$crate::block_timestamp().to_be_bytes())) % 100;
            let mut cum = 0u16;
            let mut found = false;
            $(
                if !found {
                    if choice < cum + $weight {
                        paste::paste! {
                            [<feature_set_ $feature_name:lower>](true);
                        }
                        found = true;
                    } else {
                        cum += $weight;
                    }
                }
            )*
        }
    };
}

#[macro_export]
macro_rules! FEATURE_MATCH {
    ($($feature:ident => $expr:expr),+ , * => $default:expr $(,)?) => {
        paste::paste! {
            $(
                if [<feature_is_ $feature:lower>]() {
                    $expr
                } else
            )+
            {
                $default
            }
        }
    };
    ($($feature:ident => $expr:expr),+ , _ => $default:expr $(,)?) => {
        FEATURE_MATCH! {
            $($feature => $expr,)+
            * => $default
        }
    };
    ($($feature:ident => $expr:expr),+ $(,)?) => {
        FEATURE_MATCH! {
            $($feature => $expr,)+
            * => ()
        }
    };
}

#[cfg(all(test, feature = "std"))]
mod test {
    use bobcat_entry::entry_host::set_block_timestamp;

    bobcat_features!(test123, swag);

    #[test]
    fn test_feature() {
        assert_eq!(2, _FEATURE_COUNT);
        feature_set_test123(true);
        assert!(FEATURE_IF_TEST123!({ true } else { false }));
        feature_set_test123(false);
        assert!(!feature_is_test123());
        let mut test123_count = 0;
        let mut swag_count = 0;
        let mut else_count = 0;
        for i in 0..100_000 {
            set_block_timestamp(i);
            FEATURE_PICK!(test123(40), swag(30));
            FEATURE_MATCH! {
                test123 => test123_count += 1,
                swag => swag_count += 1,
                * => else_count += 1
            };
            feature_set_test123(false);
            feature_set_swag(false);
        }
        assert!(40_000 >= test123_count || 39_900 <= test123_count);
        assert!(30_000 >= swag_count || 29_900 <= swag_count);
        assert!(50_000 >= else_count || 49_900 <= else_count);
    }
}
