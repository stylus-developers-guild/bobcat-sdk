use bobcat_maths::mutant_killer_internals::{
    wrapping_add_b, wrapping_div_b, wrapping_mod_b, wrapping_mul_b, wrapping_sub_b,
};

fn mask<const C: usize>() -> u128 {
    assert!(C > 0 && C <= 16);
    u128::MAX >> (128 - (C as u32 * 8))
}

fn to_be_bytes<const C: usize>(value: u128) -> [u8; C] {
    assert!(C > 0 && C <= 16);
    assert!(value <= mask::<C>());
    let mut out = [0u8; C];
    let bytes = value.to_be_bytes();
    out.copy_from_slice(&bytes[16 - C..]);
    out
}

macro_rules! generate_wrapping_add_tests {
    ($($name:ident: $c:literal, $lhs:expr, $rhs:expr),+ $(,)?) => {
        $(
            #[test]
            fn $name() {
                const C: usize = $c;
                let lhs_value: u128 = $lhs;
                let rhs_value: u128 = $rhs;
                let lhs = to_be_bytes::<C>(lhs_value);
                let rhs = to_be_bytes::<C>(rhs_value);
                let expected = to_be_bytes::<C>((lhs_value + rhs_value) & mask::<C>());
                assert_eq!(
                    wrapping_add_b::<C>(&lhs, &rhs),
                    expected,
                    "lhs={:#x}, rhs={:#x}",
                    lhs_value,
                    rhs_value
                );
            }
        )+
    };
}

macro_rules! generate_wrapping_sub_tests {
    ($($name:ident: $c:literal, $lhs:expr, $rhs:expr),+ $(,)?) => {
        $(
            #[test]
            fn $name() {
                const C: usize = $c;
                let lhs_value: u128 = $lhs;
                let rhs_value: u128 = $rhs;
                let lhs = to_be_bytes::<C>(lhs_value);
                let rhs = to_be_bytes::<C>(rhs_value);
                let expected = to_be_bytes::<C>((lhs_value.wrapping_sub(rhs_value)) & mask::<C>());
                assert_eq!(
                    wrapping_sub_b::<C>(&lhs, &rhs),
                    expected,
                    "lhs={:#x}, rhs={:#x}",
                    lhs_value,
                    rhs_value
                );
            }
        )+
    };
}

macro_rules! generate_wrapping_mul_tests {
    ($($name:ident: $c:literal, $lhs:expr, $rhs:expr),+ $(,)?) => {
        $(
            #[test]
            fn $name() {
                const C: usize = $c;
                let lhs_value: u128 = $lhs;
                let rhs_value: u128 = $rhs;
                let lhs = to_be_bytes::<C>(lhs_value);
                let rhs = to_be_bytes::<C>(rhs_value);
                let expected = to_be_bytes::<C>((lhs_value.wrapping_mul(rhs_value)) & mask::<C>());
                assert_eq!(
                    wrapping_mul_b::<C>(&lhs, &rhs),
                    expected,
                    "lhs={:#x}, rhs={:#x}",
                    lhs_value,
                    rhs_value
                );
            }
        )+
    };
}

macro_rules! generate_wrapping_div_tests {
    ($($name:ident: $c:literal, $lhs:expr, $rhs:expr),+ $(,)?) => {
        $(
            #[test]
            fn $name() {
                const C: usize = $c;
                let lhs_value: u128 = $lhs;
                let rhs_value: u128 = $rhs;
                let numerator = to_be_bytes::<C>(lhs_value);
                let denominator = to_be_bytes::<C>(rhs_value.min(mask::<C>()));
                let expected = if rhs_value == 0 {
                    [0u8; C]
                } else {
                    to_be_bytes::<C>(lhs_value / rhs_value)
                };
                assert_eq!(
                    wrapping_div_b::<C>(&numerator, &denominator),
                    expected,
                    "lhs={:#x}, rhs={:#x}",
                    lhs_value,
                    rhs_value
                );
            }
        )+
    };
}

macro_rules! generate_wrapping_mod_tests {
    ($($name:ident: $c:literal, $lhs:expr, $rhs:expr),+ $(,)?) => {
        $(
            #[test]
            fn $name() {
                const C: usize = $c;
                let lhs_value: u128 = $lhs;
                let rhs_value: u128 = $rhs;
                let numerator = to_be_bytes::<C>(lhs_value);
                let denominator = to_be_bytes::<C>(rhs_value.min(mask::<C>()));
                let expected = if rhs_value == 0 {
                    [0u8; C]
                } else {
                    to_be_bytes::<C>(lhs_value % rhs_value)
                };
                assert_eq!(
                    wrapping_mod_b::<C>(&numerator, &denominator),
                    expected,
                    "lhs={:#x}, rhs={:#x}",
                    lhs_value,
                    rhs_value
                );
            }
        )+
    };
}

generate_wrapping_add_tests! {
    wrapping_add_b_c4_case00: 4, 0u128, 0u128,
    wrapping_add_b_c4_case01: 4, 1u128, 0u128,
    wrapping_add_b_c4_case02: 4, 1u128, 2u128,
    wrapping_add_b_c4_case03: 4, 15u128, 30u128,
    wrapping_add_b_c4_case04: 4, 0xFFFF_FFFFu128, 1u128,
    wrapping_add_b_c4_case05: 4, 0x8000_0000u128, 0x7FFF_FFFFu128,
    wrapping_add_b_c4_case06: 4, 0x1234_5678u128, 0x8765_4321u128,
    wrapping_add_b_c4_case07: 4, 0xFFFF_0000u128, 0x0000_FFFFu128,
    wrapping_add_b_c4_case08: 4, 0x0F0F_F0F0u128, 0xF0F0_0F0Fu128,
    wrapping_add_b_c4_case09: 4, 0xDEAD_BEEFu128, 0x1111_2222u128,
    wrapping_add_b_c5_case10: 5, 0x0000_0000_00u128, 0x0000_0000_00u128,
    wrapping_add_b_c5_case11: 5, 0x0000_0000_01u128, 0x0000_0000_01u128,
    wrapping_add_b_c5_case12: 5, 0x00FF_FFFF_F0u128, 0x0000_0000_0Fu128,
    wrapping_add_b_c5_case13: 5, 0x0100_0000_00u128, 0x0000_0000_01u128,
    wrapping_add_b_c5_case14: 5, 0x0FFF_FFFF_FFu128, 0x0000_0000_01u128,
    wrapping_add_b_c5_case15: 5, 0x0080_0000_00u128, 0x0080_0000_00u128,
    wrapping_add_b_c5_case16: 5, 0x0001_2345_67u128, 0x0000_7654_32u128,
    wrapping_add_b_c5_case17: 5, 0x0ABC_DE12_34u128, 0x0012_3456_78u128,
    wrapping_add_b_c5_case18: 5, 0x00F0_0F0F_0Fu128, 0x000F_F0F0_F0u128,
    wrapping_add_b_c5_case19: 5, 0x0EED_CBA9_87u128, 0x0011_2233_44u128,
}

generate_wrapping_sub_tests! {
    wrapping_sub_b_c4_case20: 4, 0u128, 0u128,
    wrapping_sub_b_c4_case21: 4, 1u128, 0u128,
    wrapping_sub_b_c4_case22: 4, 2u128, 1u128,
    wrapping_sub_b_c4_case23: 4, 30u128, 15u128,
    wrapping_sub_b_c4_case24: 4, 0x0000_0000u128, 1u128,
    wrapping_sub_b_c4_case25: 4, 0x7FFF_FFFFu128, 0x8000_0000u128,
    wrapping_sub_b_c4_case26: 4, 0x8765_4321u128, 0x1234_5678u128,
    wrapping_sub_b_c4_case27: 4, 0x0000_FFFFu128, 0xFFFF_0000u128,
    wrapping_sub_b_c4_case28: 4, 0xF0F0_0F0Fu128, 0x0F0F_F0F0u128,
    wrapping_sub_b_c4_case29: 4, 0x1111_2222u128, 0xDEAD_BEEFu128,
    wrapping_sub_b_c5_case30: 5, 0x0000_0000_00u128, 0x0000_0000_00u128,
    wrapping_sub_b_c5_case31: 5, 0x0000_0000_01u128, 0x0000_0000_00u128,
    wrapping_sub_b_c5_case32: 5, 0x0000_0000_0Fu128, 0x0000_0000_0Eu128,
    wrapping_sub_b_c5_case33: 5, 0x0000_0000_10u128, 0x0000_0000_01u128,
    wrapping_sub_b_c5_case34: 5, 0x0000_0000_00u128, 0x0000_0000_01u128,
    wrapping_sub_b_c5_case35: 5, 0x0080_0000_00u128, 0x0080_0000_00u128,
    wrapping_sub_b_c5_case36: 5, 0x0000_7654_32u128, 0x0001_2345_67u128,
    wrapping_sub_b_c5_case37: 5, 0x0012_3456_78u128, 0x0ABC_DE12_34u128,
    wrapping_sub_b_c5_case38: 5, 0x000F_F0F0_F0u128, 0x00F0_0F0F_0Fu128,
    wrapping_sub_b_c5_case39: 5, 0x0011_2233_44u128, 0x0EED_CBA9_87u128,
}

generate_wrapping_mul_tests! {
    wrapping_mul_b_c4_case40: 4, 0u128, 0u128,
    wrapping_mul_b_c4_case41: 4, 1u128, 0u128,
    wrapping_mul_b_c4_case42: 4, 1u128, 1u128,
    wrapping_mul_b_c4_case43: 4, 2u128, 2u128,
    wrapping_mul_b_c4_case44: 4, 15u128, 30u128,
    wrapping_mul_b_c4_case45: 4, 0xFFFF_FFFFu128, 2u128,
    wrapping_mul_b_c4_case46: 4, 0x8000_0000u128, 2u128,
    wrapping_mul_b_c4_case47: 4, 0x1234_5678u128, 3u128,
    wrapping_mul_b_c4_case48: 4, 0x0000_FFFFu128, 0x0000_FFFFu128,
    wrapping_mul_b_c4_case49: 4, 0xDEAD_BEEFu128, 0x1111_2222u128,
    wrapping_mul_b_c5_case50: 5, 0x0000_0000_00u128, 0x0000_0000_00u128,
    wrapping_mul_b_c5_case51: 5, 0x0000_0000_01u128, 0x0000_0000_00u128,
    wrapping_mul_b_c5_case52: 5, 0x0000_0000_01u128, 0x0000_0000_01u128,
    wrapping_mul_b_c5_case53: 5, 0x0000_0000_02u128, 0x0000_0000_02u128,
    wrapping_mul_b_c5_case54: 5, 0x0000_0000_0Fu128, 0x0000_0000_0Fu128,
    wrapping_mul_b_c5_case55: 5, 0x0080_0000_00u128, 0x0000_0000_02u128,
    wrapping_mul_b_c5_case56: 5, 0x0001_2345_67u128, 0x0000_0000_03u128,
    wrapping_mul_b_c5_case57: 5, 0x0ABC_DE12_34u128, 0x0000_0000_04u128,
    wrapping_mul_b_c5_case58: 5, 0x00F0_0F0F_0Fu128, 0x0000_0000_05u128,
    wrapping_mul_b_c5_case59: 5, 0x0EED_CBA9_87u128, 0x0000_0000_06u128,
}

generate_wrapping_div_tests! {
    wrapping_div_b_c4_case60: 4, 0u128, 1u128,
    wrapping_div_b_c4_case61: 4, 1u128, 1u128,
    wrapping_div_b_c4_case62: 4, 2u128, 1u128,
    wrapping_div_b_c4_case63: 4, 30u128, 2u128,
    wrapping_div_b_c4_case64: 4, 0xFFFF_FFFFu128, 2u128,
    wrapping_div_b_c4_case65: 4, 0x8000_0000u128, 0x7FFF_FFFFu128,
    wrapping_div_b_c4_case66: 4, 0x1234_5678u128, 3u128,
    wrapping_div_b_c4_case67: 4, 0xFFFF_0000u128, 0x0000_FFFFu128,
    wrapping_div_b_c4_case68: 4, 0x0F0F_F0F0u128, 0xF0F0_0F0Fu128,
    wrapping_div_b_c4_case69: 4, 0xDEAD_BEEFu128, 0x1111_2222u128,
    wrapping_div_b_c5_case70: 5, 0x0000_0000_00u128, 1u128,
    wrapping_div_b_c5_case71: 5, 0x0000_0000_01u128, 1u128,
    wrapping_div_b_c5_case72: 5, 0x00FF_FFFF_F0u128, 0x0000_0000_0Fu128,
    wrapping_div_b_c5_case73: 5, 0x0100_0000_00u128, 0x0000_0000_01u128,
    wrapping_div_b_c5_case74: 5, 0x0FFF_FFFF_FFu128, 0x0000_0000_01u128,
    wrapping_div_b_c5_case75: 5, 0x0080_0000_00u128, 0x0080_0000_00u128,
    wrapping_div_b_c5_case76: 5, 0x0001_2345_67u128, 0x0000_7654_32u128,
    wrapping_div_b_c5_case77: 5, 0x0ABC_DE12_34u128, 0x0012_3456_78u128,
    wrapping_div_b_c5_case78: 5, 0x00F0_0F0F_0Fu128, 0x000F_F0F0_F0u128,
    wrapping_div_b_c5_case79: 5, 0x0EED_CBA9_87u128, 0x0011_2233_44u128,
}

generate_wrapping_mod_tests! {
    wrapping_mod_b_c4_case80: 4, 0u128, 1u128,
    wrapping_mod_b_c4_case81: 4, 1u128, 1u128,
    wrapping_mod_b_c4_case82: 4, 2u128, 1u128,
    wrapping_mod_b_c4_case83: 4, 30u128, 2u128,
    wrapping_mod_b_c4_case84: 4, 0xFFFF_FFFFu128, 2u128,
    wrapping_mod_b_c4_case85: 4, 0x8000_0000u128, 0x7FFF_FFFFu128,
    wrapping_mod_b_c4_case86: 4, 0x1234_5678u128, 3u128,
    wrapping_mod_b_c4_case87: 4, 0xFFFF_0000u128, 0x0000_FFFFu128,
    wrapping_mod_b_c4_case88: 4, 0x0F0F_F0F0u128, 0xF0F0_0F0Fu128,
    wrapping_mod_b_c4_case89: 4, 0xDEAD_BEEFu128, 0x1111_2222u128,
    wrapping_mod_b_c5_case90: 5, 0x0000_0000_00u128, 1u128,
    wrapping_mod_b_c5_case91: 5, 0x0000_0000_01u128, 1u128,
    wrapping_mod_b_c5_case92: 5, 0x00FF_FFFF_F0u128, 0x0000_0000_0Fu128,
    wrapping_mod_b_c5_case93: 5, 0x0100_0000_00u128, 0x0000_0000_01u128,
    wrapping_mod_b_c5_case94: 5, 0x0FFF_FFFF_FFu128, 0x0000_0000_01u128,
    wrapping_mod_b_c5_case95: 5, 0x0080_0000_00u128, 0x0080_0000_00u128,
    wrapping_mod_b_c5_case96: 5, 0x0001_2345_67u128, 0x0000_7654_32u128,
    wrapping_mod_b_c5_case97: 5, 0x0ABC_DE12_34u128, 0x0012_3456_78u128,
    wrapping_mod_b_c5_case98: 5, 0x00F0_0F0F_0Fu128, 0x000F_F0F0_F0u128,
    wrapping_mod_b_c5_case99: 5, 0x0EED_CBA9_87u128, 0x0011_2233_44u128,
}
