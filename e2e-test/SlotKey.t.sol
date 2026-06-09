// SPDX-License-Identifier: MIT
pragma solidity 0.8.30;

import "forge-std/console.sol";

import {Test} from "forge-std/Test.sol";

import {IArbFoundry} from "./IArbFoundry.sol";

interface Impl1 {
    function test(uint256 x, uint256 y) external returns (uint256);
}

contract Impl2 {
    uint256 __unused1;
    uint256 __unused2;
    mapping(uint256 => mapping(uint256 => uint256)) swag;

    function set(uint256 x, uint256 y, uint256 z) external {
        swag[x][y] = z;
    }
}

contract Proxy {
    bytes32 constant SLOT_IMPL1 = bytes32(uint256(keccak256(abi.encodePacked("testing.slot.1"))) - 1);
    bytes32 constant SLOT_IMPL2 = bytes32(uint256(keccak256(abi.encodePacked("testing.slot.2"))) - 1);

    constructor(address addr1, address addr2) {
        bytes32 slotImpl1 = SLOT_IMPL1;
        bytes32 slotImpl2 = SLOT_IMPL2;
        assembly {
            sstore(slotImpl1, addr1)
            sstore(slotImpl2, addr2)
        }
    }

    fallback() external {
        bytes32 slotImpl1 = SLOT_IMPL1;
        bytes32 slotImpl2 = SLOT_IMPL2;
        address addrImpl1;
        address addrImpl2;
        assembly {
            addrImpl1 := sload(slotImpl1)
            addrImpl2 := sload(slotImpl2)
        }
        address impl;
        if (msg.sig == Impl2.set.selector) {
            impl = addrImpl2;
        } else {
            impl = addrImpl1;
        }
        assembly {
            calldatacopy(0, 0, calldatasize())
            let result := delegatecall(gas(), impl, 0, calldatasize(), 0, 0)
            returndatacopy(0, 0, returndatasize())
            switch result
            case 0 { revert(0, returndatasize()) }
            default { return(0, returndatasize()) }
        }
    }
}

contract SlotKey is Test {
    address proxy;

    function setUp() public {
        address impl1 = IArbFoundry(address(vm)).deployStylusCode(
            "e2e-test/slot-key.wasm"
        );
        address impl2 = address(new Impl2());
        proxy = address(new Proxy(impl1, impl2));
    }

    function testFuzz_rightSlot(uint256 x, uint256 y, uint256 z) public {
        Impl2(proxy).set(x, y, z);
        assertEq(z, Impl1(proxy).test(x, y));
    }
}
