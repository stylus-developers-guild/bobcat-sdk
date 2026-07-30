#![cfg_attr(not(feature = "std"), no_std)]

pub use bobcat_storage::{
    U, const_keccak256_two_off_curve, keccak256, storage_load, storage_store,
};

pub use bobcat_entry::block_timestamp;

pub use bobcat_interfaces::superposition::make_fn_features;

pub use bobcat_call::static_call_word;

pub use paste::paste;

#[macro_export]
macro_rules! BOBCAT_FEATURES {
    ($($feature_name:ident),* $(,)?) => {
        pub const _FEATURE_COUNT: u8 = 0 $(+ { let _ = stringify!($feature_name); 1 })*;

        // We need the first slot of the word to be the length:
        const _: [(); 0] = [(); (_FEATURE_COUNT < 255) as usize - 1];

        $(
            $crate::paste! {
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

        #[allow(unused)]
        pub fn feature_pack() -> $crate::U {
            let mut r = $crate::U::default();
            let mut i = 1;
            $(
                $crate::paste! {
                    if [<feature_is_ $feature_name:lower>]() {
                        let byte_index = 31 - (i / 8);
                        let bit_position = i % 8;
                        r[byte_index] |= 1 << bit_position;
                    }
                    i += 1;
                }
            )*
            let _ = i;
            r
        }
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
                        $crate::paste! {
                            [<feature_set_ $feature_name:lower>](true);
                        }
                        found = true;
                    } else {
                        cum += $weight;
                    }
                }
            )*
            let _ = cum;
            let _ = found;
        }
    };
}

#[macro_export]
macro_rules! FEATURE_MATCH {
    ($($feature:ident => $expr:expr),+ , * => $default:expr $(,)?) => {
        $crate::paste! {
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

#[macro_export]
macro_rules! FEATURE_COPY {
    ($address:expr, $($feature_name:ident),* $(,)?) => {
        #[cfg(not(any(
            target_arch = "riscv32",
            all(target_family = "wasm", target_os = "unknown"))
        ))]
        {
            // This is a no-op on this host! We assume someone is running this with a
            // testing harness.
        }
        #[cfg(any(
            target_arch = "riscv32",
            all(target_family = "wasm", target_os = "unknown")
        ))]
        {
            let (rc, r) = $crate::static_call_word(
                $address,
                &$crate::make_fn_features(),
                u64::MAX,
                0
            );
            assert!(rc, "features revert");
            let remote_count = r[0];
            const COUNT: u8 = 0 $(+ { let _ = stringify!($feature_name); 1 })*;
            assert_eq!(
                remote_count, COUNT,
                "features {remote_count} != {COUNT}"
            );
            let mut i = 1;
            $(
                $crate::paste! {
                    let byte_index = 31 - (i / 8);
                    let bit_position = i % 8;
                    $crate::storage_store(
                        &$crate::const_keccak256_two_off_curve(
                            b"bobcat.features.",
                            stringify!($feature_name).as_bytes()
                        ),
                        &$crate::U::from((r[byte_index] & (1 << bit_position)) != 0)
                    );
                    i += 1;
                }
            )*
            let _ = i;
        }
    };
}

#[macro_export]
macro_rules! FEATURE_COPY_NON_ZEROES {
    ($address:expr, $($feature_name:ident),* $(,)?) => {{
        #[cfg(not(any(
            target_arch = "riscv32",
            all(target_family = "wasm", target_os = "unknown"))
        ))]
        {
            // This is a no-op on this host! We assume someone is running this with a
            // testing harness.
        }
        #[cfg(any(
            target_arch = "riscv32",
            all(target_family = "wasm", target_os = "unknown")
        ))]
        {
            let (rc, r) = $crate::static_call_word(
                $address,
                &$crate::make_fn_features(),
                u64::MAX,
                0
            );
            assert!(rc, "features revert");
            let remote_count = r[0];
            const COUNT: u8 = 0 $(+ { let _ = stringify!($feature_name); 1 })*;
            assert_eq!(
                remote_count, COUNT,
                "features {remote_count} != {COUNT}"
            );
            let mut i = 1;
            $(
                $crate::paste! {
                    let byte_index = 31 - (i / 8);
                    let bit_position = i % 8;
                    let setting = (r[byte_index] & (1 << bit_position)) != 0;
                    if setting {
                    $crate::storage_store(
                        &$crate::const_keccak256_two_off_curve(
                            b"bobcat.features.",
                            stringify!($feature_name).as_bytes()
                        ),
                        &$crate::U::from(setting)
                    );
                    }
                    i += 1;
                }
            )*
            let _ = i;
        }
    }};
}

#[macro_export]
macro_rules! FEATURE_PACK {
    ($($feature_name:ident),* $(,)?) => {
        {
            let mut r = $crate::U::default();
            #[allow(unused_assignments)]
            let mut i = 1;
            const COUNT: u8 = 0 $(+ { let _ = stringify!($feature_name); 1 })*;
            r[0] = COUNT;
            $(
                $crate::paste! {
                    if [<feature_is_ $feature_name:lower>]() {
                        let byte_index = 31 - (i / 8);
                        let bit_position = i % 8;
                        r[byte_index] |= 1 << bit_position;
                    }
                    i += 1;
                }
            )*
            let _ = i;
            r
        }
    };
}

#[cfg(all(test, feature = "std"))]
mod test_1 {
    use bobcat_host::{set_block_timestamp, storage_clear};

    use super::U;

    BOBCAT_FEATURES!(test123, swag);

    #[test]
    fn test_features_1() {
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
        feature_set_test123(true);
        feature_set_swag(true);
        assert_eq!(U::from(6u32), feature_pack());
        feature_set_swag(false);
        assert_eq!(U::from(2u32), feature_pack());
        storage_clear();
    }
}

#[cfg(all(test, feature = "std", feature = "alloy-enabled"))]
mod test_2 {
    use std::str::FromStr;

    use bobcat_entry::U;

    use bobcat_host::storage_clear;

    BOBCAT_FEATURES!(
        F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12, F13, F14, F15, F16, F17, F18, F19, F20,
        F21, F22, F23, F24, F25, F26, F27, F28, F29, F30, F31, F32, F33, F34, F35, F36, F37, F38,
        F39, F40, F41, F42, F43, F44, F45, F46, F47, F48, F49, F50, F51, F52, F53, F54, F55, F56,
        F57, F58, F59, F60, F61, F62, F63, F64, F65, F66, F67, F68, F69, F70, F71, F72, F73, F74,
        F75, F76, F77, F78, F79, F80, F81, F82, F83, F84, F85, F86, F87, F88, F89, F90, F91, F92,
        F93, F94, F95, F96, F97, F98, F99, F100, F101, F102, F103, F104, F105, F106, F107, F108,
        F109, F110, F111, F112, F113, F114, F115, F116, F117, F118, F119, F120, F121, F122, F123,
        F124, F125, F126, F127, F128, F129, F130, F131, F132, F133, F134, F135, F136, F137, F138,
        F139, F140, F141, F142, F143, F144, F145, F146, F147, F148, F149, F150, F151, F152, F153,
        F154, F155, F156, F157, F158, F159, F160, F161, F162, F163, F164, F165, F166, F167, F168,
        F169, F170, F171, F172, F173, F174, F175, F176, F177, F178, F179, F180, F181, F182, F183,
        F184, F185, F186, F187, F188, F189, F190, F191, F192, F193, F194, F195, F196, F197, F198,
        F199, F200, F201, F202, F203, F204, F205, F206, F207, F208, F209, F210, F211, F212, F213,
        F214, F215, F216, F217, F218, F219, F220, F221, F222, F223, F224, F225, F226, F227, F228,
        F229, F230, F231, F232, F233, F234, F235, F236, F237, F238, F239, F240, F241, F242, F243,
        F244, F245, F246, F247, F248, F249, F250, F251, F252, F253, F254,
    );

    #[test]
    fn test() {
        feature_set_f1(true);
        feature_set_f2(true);
        feature_set_f3(true);
        feature_set_f4(true);
        feature_set_f5(true);
        feature_set_f6(true);
        feature_set_f7(true);
        feature_set_f8(true);
        feature_set_f9(true);
        feature_set_f10(true);
        feature_set_f11(true);
        feature_set_f12(true);
        feature_set_f13(true);
        feature_set_f14(true);
        feature_set_f15(true);
        feature_set_f16(true);
        feature_set_f17(true);
        feature_set_f18(true);
        feature_set_f19(true);
        feature_set_f20(true);
        feature_set_f21(true);
        feature_set_f22(true);
        feature_set_f23(true);
        feature_set_f24(true);
        feature_set_f25(true);
        feature_set_f26(true);
        feature_set_f27(true);
        feature_set_f28(true);
        feature_set_f29(true);
        feature_set_f30(true);
        feature_set_f31(true);
        feature_set_f32(true);
        feature_set_f33(true);
        feature_set_f34(true);
        feature_set_f35(true);
        feature_set_f36(true);
        feature_set_f37(true);
        feature_set_f38(true);
        feature_set_f39(true);
        feature_set_f40(true);
        feature_set_f41(true);
        feature_set_f42(true);
        feature_set_f43(true);
        feature_set_f44(true);
        feature_set_f45(true);
        feature_set_f46(true);
        feature_set_f47(true);
        feature_set_f48(true);
        feature_set_f49(true);
        feature_set_f50(true);
        feature_set_f51(true);
        feature_set_f52(true);
        feature_set_f53(true);
        feature_set_f54(true);
        feature_set_f55(true);
        feature_set_f56(true);
        feature_set_f57(true);
        feature_set_f58(true);
        feature_set_f59(true);
        feature_set_f60(true);
        feature_set_f61(true);
        feature_set_f62(true);
        feature_set_f63(true);
        feature_set_f64(true);
        feature_set_f65(true);
        feature_set_f66(true);
        feature_set_f67(true);
        feature_set_f68(true);
        feature_set_f69(true);
        feature_set_f70(true);
        feature_set_f71(true);
        feature_set_f72(true);
        feature_set_f73(true);
        feature_set_f74(true);
        feature_set_f75(true);
        feature_set_f76(true);
        feature_set_f77(true);
        feature_set_f78(true);
        feature_set_f79(true);
        feature_set_f80(true);
        feature_set_f81(true);
        feature_set_f82(true);
        feature_set_f83(true);
        feature_set_f84(true);
        feature_set_f85(true);
        feature_set_f86(true);
        feature_set_f87(true);
        feature_set_f88(true);
        feature_set_f89(true);
        feature_set_f90(true);
        feature_set_f91(true);
        feature_set_f92(true);
        feature_set_f93(true);
        feature_set_f94(true);
        feature_set_f95(true);
        feature_set_f96(true);
        feature_set_f97(true);
        feature_set_f98(true);
        feature_set_f99(true);
        feature_set_f100(true);
        feature_set_f101(true);
        feature_set_f102(true);
        feature_set_f103(true);
        feature_set_f104(true);
        feature_set_f105(true);
        feature_set_f106(true);
        feature_set_f107(true);
        feature_set_f108(true);
        feature_set_f109(true);
        feature_set_f110(true);
        feature_set_f111(true);
        feature_set_f112(true);
        feature_set_f113(true);
        feature_set_f114(true);
        feature_set_f115(true);
        feature_set_f116(true);
        feature_set_f117(true);
        feature_set_f118(true);
        feature_set_f119(true);
        feature_set_f120(true);
        feature_set_f121(true);
        feature_set_f122(true);
        feature_set_f123(true);
        feature_set_f124(true);
        feature_set_f125(true);
        feature_set_f126(true);
        feature_set_f127(true);
        feature_set_f128(true);
        feature_set_f129(true);
        feature_set_f130(true);
        feature_set_f131(true);
        feature_set_f132(true);
        feature_set_f133(true);
        feature_set_f134(true);
        feature_set_f135(true);
        feature_set_f136(true);
        feature_set_f137(true);
        feature_set_f138(true);
        feature_set_f139(true);
        feature_set_f140(true);
        feature_set_f141(true);
        feature_set_f142(true);
        feature_set_f143(true);
        feature_set_f144(true);
        feature_set_f145(true);
        feature_set_f146(true);
        feature_set_f147(true);
        feature_set_f148(true);
        feature_set_f149(true);
        feature_set_f150(true);
        feature_set_f151(true);
        feature_set_f152(true);
        feature_set_f153(true);
        feature_set_f154(true);
        feature_set_f155(true);
        feature_set_f156(true);
        feature_set_f157(true);
        feature_set_f158(true);
        feature_set_f159(true);
        feature_set_f160(true);
        feature_set_f161(true);
        feature_set_f162(true);
        feature_set_f163(true);
        feature_set_f164(true);
        feature_set_f165(true);
        feature_set_f166(true);
        feature_set_f167(true);
        feature_set_f168(true);
        feature_set_f169(true);
        feature_set_f170(true);
        feature_set_f171(true);
        feature_set_f172(true);
        feature_set_f173(true);
        feature_set_f174(true);
        feature_set_f175(true);
        feature_set_f176(true);
        feature_set_f177(true);
        feature_set_f178(true);
        feature_set_f179(true);
        feature_set_f180(true);
        feature_set_f181(true);
        feature_set_f182(true);
        feature_set_f183(true);
        feature_set_f184(true);
        feature_set_f185(true);
        feature_set_f186(true);
        feature_set_f187(true);
        feature_set_f188(true);
        feature_set_f189(true);
        feature_set_f190(true);
        feature_set_f191(true);
        feature_set_f192(true);
        feature_set_f193(true);
        feature_set_f194(true);
        feature_set_f195(true);
        feature_set_f196(true);
        feature_set_f197(true);
        feature_set_f198(true);
        feature_set_f199(true);
        feature_set_f200(true);
        feature_set_f201(true);
        feature_set_f202(true);
        feature_set_f203(true);
        feature_set_f204(true);
        feature_set_f205(true);
        feature_set_f206(true);
        feature_set_f207(true);
        feature_set_f208(true);
        feature_set_f209(true);
        feature_set_f210(true);
        feature_set_f211(true);
        feature_set_f212(true);
        feature_set_f213(true);
        feature_set_f214(true);
        feature_set_f215(true);
        feature_set_f216(true);
        feature_set_f217(true);
        feature_set_f218(true);
        feature_set_f219(true);
        feature_set_f220(true);
        feature_set_f221(true);
        feature_set_f222(true);
        feature_set_f223(true);
        feature_set_f224(true);
        feature_set_f225(true);
        feature_set_f226(true);
        feature_set_f227(true);
        feature_set_f228(true);
        feature_set_f229(true);
        feature_set_f230(true);
        feature_set_f231(true);
        feature_set_f232(true);
        feature_set_f233(true);
        feature_set_f234(true);
        feature_set_f235(true);
        feature_set_f236(true);
        feature_set_f237(true);
        feature_set_f238(true);
        feature_set_f239(true);
        feature_set_f240(true);
        feature_set_f241(true);
        feature_set_f242(true);
        feature_set_f243(true);
        feature_set_f244(true);
        feature_set_f245(true);
        feature_set_f246(true);
        feature_set_f247(true);
        feature_set_f248(true);
        feature_set_f249(true);
        feature_set_f250(true);
        feature_set_f251(true);
        feature_set_f252(true);
        feature_set_f253(true);
        feature_set_f254(true);
        assert_eq!(
            U::from_str(
                "57896044618658097711785492504343953926634992332820282019728792003956564819966"
            )
            .unwrap(),
            feature_pack()
        );
        storage_clear();
    }
}
