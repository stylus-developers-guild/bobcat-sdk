// SPDX-License-Identifier: MIT
pragma solidity ^0.8.30;

import {Vm, Test} from "forge-std/Test.sol";

import {IArbFoundry} from "./IArbFoundry.sol";

contract PrecompilesXz is Test {
    address xz = 0xC640A98Ea2809dC65aD58385BBdD9038c529d5e5;

    function setUp() public {
        vm.etch(
            xz,
            IArbFoundry(address(vm)).deployStylusCode(
                "e2e-test/superposition-precompiles/precompiles-xz.wasm"
            ).code
        );
        // We need to see if xz exists on the local machine first:
        string[] memory ffiIn = new string[](2);
        ffiIn[0] = "xz";
        ffiIn[1] = "--help";
        vm.ffi(ffiIn);
        // And we need to check if xxd is supported too:
        ffiIn = new string[](3);
        ffiIn[0] = "xxd";
        ffiIn[1] = "-ps";
        ffiIn[2] = "-c0";
        vm.ffi(ffiIn);
        // Let's try to invoke the testing tool we have here to generate the calldata:
        ffiIn = new string[](2);
        ffiIn[0] = "./e2e-test/gen-xz-cd.sh";
        ffiIn[1] = "123";
        bytes memory out = vm.ffi(ffiIn);
        assertNotEq("", out);
    }

    function testFuzz_compressionConsistency(uint256 seed) public {
        string[] memory ffiIn = new string[](2);
        ffiIn[0] = "./e2e-test/gen-xz-cd.sh";
        ffiIn[1] = vm.toString(seed);
        bytes memory compressed = vm.ffi(ffiIn);
        ffiIn[0] = "./e2e-test/gen-xz-pre.sh";
        bytes memory pre = vm.ffi(ffiIn);
        (bool rc, bytes memory out) = xz.call(compressed);
        assertEq(pre, out);
    }
}
