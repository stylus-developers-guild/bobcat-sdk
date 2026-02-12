// SPDX-License-Identifier: MIT
pragma solidity ^0.8.30;

import {Test} from "forge-std/Test.sol";

import {IArbFoundry} from "./IArbFoundry.sol";

interface IMathExample {
    function getConstant() external view returns (uint256);
    function add(uint256 a, uint256 b) external pure returns (uint256);
    function mul(uint256 a, uint256 b) external pure returns (uint256);
}

contract MathExampleTest is Test {
    IMathExample math;

    function setUp() public {
        math = IMathExample(IArbFoundry(address(vm)).deployStylusCode(
            "e2e-test/math-example.wasm"
        ));
    }

    function testGetConstant() public view {
        // maths_zone!("2**8 + 5") = 256 + 5 = 261
        assertEq(math.getConstant(), 261);
    }

    function testAdd() public view {
        assertEq(math.add(3, 5), 8);
    }

    function testAddLargeNumbers() public view {
        assertEq(math.add(1000000, 2000000), 3000000);
    }

    function testMul() public view {
        assertEq(math.mul(4, 7), 28);
    }

    function testMulByZero() public view {
        assertEq(math.mul(999, 0), 0);
    }

    function testMulByOne() public view {
        assertEq(math.mul(42, 1), 42);
    }
}
