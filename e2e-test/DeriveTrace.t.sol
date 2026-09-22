// SPDX-License-Identifier: MIT
pragma solidity 0.8.30;

import {Test} from "forge-std/Test.sol";

import {IArbFoundry} from "./IArbFoundry.sol";

interface ISafeERC20 {
    function test(address) external pure returns (bool);
}

contract DeriveTrace is Test {
    address safeErc20;

    function setUp() public {
        safeErc20 = IArbFoundry(address(vm)).deployStylusCode(
            "e2e-test/derive-trace.wasm"
        );
    }

    function test_trace() external {
        (bool rc,) = safeErc20.call("");
        assert(rc);
    }
}
