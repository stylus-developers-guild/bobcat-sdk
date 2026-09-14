
use array_concat::concat_arrays;

use bobcat_cd::leftpad_addr;

use crate::selectors;

pub use crate::{eip20::*, eip2612::*};

type Address = [u8; 20];

selectors! {
    SEL_ACCESS_CONTROLLED_REGISTRY = b"ACCESS_CONTROLLED_REGISTRY()",
    SEL_UID = b"uid()",
    SEL_TERMS = b"terms()",
    SEL_PAUSED = b"paused()",
    SEL_TOKEN_PAUSED = b"tokenPaused()",
    SEL_ORACLE_PAUSED = b"oraclePaused()",
    SEL_UI_MULTIPLIER = b"uiMultiplier()",
    SEL_NEW_UI_MULTIPLIER = b"newUIMultiplier()",
    SEL_EFFECTIVE_AT = b"effectiveAt()",
    SEL_BALANCE_OF_UI = b"balanceOfUI(address)",
    SEL_TOTAL_SUPPLY_UI = b"totalSupplyUI()",
    SEL_SUPPORTS_INTERFACE = b"supportsInterface(bytes4)",
}

pub const fn make_fn_access_controlled_registry() -> [u8; 4] {
    SEL_ACCESS_CONTROLLED_REGISTRY
}

pub const fn make_fn_uid() -> [u8; 4] {
    SEL_UID
}

pub const fn make_fn_terms() -> [u8; 4] {
    SEL_TERMS
}

pub const fn make_fn_paused() -> [u8; 4] {
    SEL_PAUSED
}

pub const fn make_fn_token_paused() -> [u8; 4] {
    SEL_TOKEN_PAUSED
}

pub const fn make_fn_oracle_paused() -> [u8; 4] {
    SEL_ORACLE_PAUSED
}

pub const fn make_fn_ui_multiplier() -> [u8; 4] {
    SEL_UI_MULTIPLIER
}

pub const fn make_fn_new_ui_multiplier() -> [u8; 4] {
    SEL_NEW_UI_MULTIPLIER
}

/// Returns the timestamp at which the scheduled UI multiplier takes effect.
pub const fn make_fn_effective_at() -> [u8; 4] {
    SEL_EFFECTIVE_AT
}

pub const fn make_fn_balance_of_ui(account: Address) -> [u8; 4 + 32] {
    concat_arrays!(SEL_BALANCE_OF_UI, leftpad_addr(account))
}

pub const fn make_fn_total_supply_ui() -> [u8; 4] {
    SEL_TOTAL_SUPPLY_UI
}

pub const fn make_fn_supports_interface(interface_id: [u8; 4]) -> [u8; 4 + 32] {
    concat_arrays!(SEL_SUPPORTS_INTERFACE, interface_id, [0u8; 28])
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::{Address as AlloyAddress, FixedBytes};
    use alloy_sol_macro::sol;
    use alloy_sol_types::SolCall;

    sol! {
        function ACCESS_CONTROLLED_REGISTRY() external view returns (address);
        function uid() external view returns (bytes32);
        function terms() external pure returns (string memory);
        function paused() external view returns (bool);
        function tokenPaused() external view returns (bool);
        function oraclePaused() external view returns (bool);
        function uiMultiplier() external view returns (uint256);
        function newUIMultiplier() external view returns (uint256);
        function effectiveAt() external view returns (uint256);
        function balanceOfUI(address account) external view returns (uint256);
        function totalSupplyUI() external view returns (uint256);
        function supportsInterface(bytes4 interfaceId) external view returns (bool);
    }

    #[test]
    fn encodes_no_argument_developer_queries() {
        assert_eq!(
            make_fn_access_controlled_registry().as_slice(),
            ACCESS_CONTROLLED_REGISTRYCall {}.abi_encode()
        );
        assert_eq!(make_fn_uid().as_slice(), uidCall {}.abi_encode());
        assert_eq!(make_fn_terms().as_slice(), termsCall {}.abi_encode());
        assert_eq!(make_fn_paused().as_slice(), pausedCall {}.abi_encode());
        assert_eq!(
            make_fn_token_paused().as_slice(),
            tokenPausedCall {}.abi_encode()
        );
        assert_eq!(
            make_fn_oracle_paused().as_slice(),
            oraclePausedCall {}.abi_encode()
        );
        assert_eq!(
            make_fn_ui_multiplier().as_slice(),
            uiMultiplierCall {}.abi_encode()
        );
        assert_eq!(
            make_fn_new_ui_multiplier().as_slice(),
            newUIMultiplierCall {}.abi_encode()
        );
        assert_eq!(
            make_fn_effective_at().as_slice(),
            effectiveAtCall {}.abi_encode()
        );
        assert_eq!(
            make_fn_total_supply_ui().as_slice(),
            totalSupplyUICall {}.abi_encode()
        );
    }

    #[test]
    fn encodes_developer_queries_with_arguments() {
        let account = [0xabu8; 20];
        assert_eq!(
            make_fn_balance_of_ui(account).as_slice(),
            balanceOfUICall {
                account: AlloyAddress::from(account),
            }
            .abi_encode()
        );

        let interface_id = [0x12, 0x34, 0x56, 0x78];
        assert_eq!(
            make_fn_supports_interface(interface_id).as_slice(),
            supportsInterfaceCall {
                interfaceId: FixedBytes::<4>::from(interface_id),
            }
            .abi_encode()
        );
    }

    #[test]
    fn exposes_standard_token_and_permit_calldata_builders() {
        let account = [0x11u8; 20];
        let spender = [0x22u8; 20];
        let amount = bobcat_maths::U::from(7u8);

        assert_eq!(
            make_fn_balance_of(account),
            crate::eip20::make_fn_balance_of(account)
        );
        assert_eq!(
            make_fn_approve(spender, &amount),
            crate::eip20::make_fn_approve(spender, &amount)
        );
        assert_eq!(
            make_fn_nonces(account),
            crate::eip2612::make_fn_nonces(account)
        );
    }
}
