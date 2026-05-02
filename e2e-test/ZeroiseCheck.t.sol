// SPDX-License-Identifier: MIT
pragma solidity ^0.8.30;

import {Vm, Test} from "forge-std/Test.sol";

import {IArbFoundry} from "./IArbFoundry.sol";

contract ZeroiseCheck is Test {
    function test_zero() public {
        address z = IArbFoundry(address(vm)).deployStylusCode(
            "e2e-test/zeroise-check.wasm"
        );
        (bool rc, bytes memory rd) = z.call("");
        for (uint i = 0; i < 1024; ++i) {
            assertEq(0, rd[i]);
        }
    }
}
