// SPDX-License-Identifier: MIT
pragma solidity ^0.8.30;

import {Vm, Test} from "forge-std/Test.sol";

/*import {IArbFoundry} from "./IArbFoundry.sol";

interface IPrecompiles {
    function ecrecover_(bytes32, uint8, bytes32, bytes32) external view returns (address);

    function createEd25519(bytes32 key, bytes memory preimage) external pure returns (
        bytes32 digestA,
        bytes32 digestB,
        bytes32 pubKey,
        bytes32 sigA,
        bytes32 sigB
    );

    function testEd25519(
        bytes32 digestA,
        bytes32 digestB,
        bytes32 pubKey,
        bytes32 sigA,
        bytes32 sigB
    ) external pure;

    function secp256r1(
        bytes32 h,
        bytes32 r,
        bytes32 s,
        bytes32 qx,
        bytes32 qy
    ) external returns (bytes32);
}*/

contract Ecrecover is Test {/*
    IPrecompiles precompiles;

    function setUp() public {
        precompiles = IPrecompiles(IArbFoundry(address(vm)).deployStylusCode(
            "e2e-test/precompiles.wasm"
        ));
        vm.etch(
            0xC3E443bE2Cfa4F41a5F5E4978D012847d355b419,
            IArbFoundry(address(vm)).deployStylusCode(
                "e2e-test/superposition-precompiles/precompiles-ed25519.wasm"
            ).code
        );
    }

    function test_fuzzEcrecover(bytes32 digest) public {
        // It isn't so important that we create a new private key for this:
        Vm.Wallet memory wallet = vm.createWallet(uint256(keccak256(bytes("1"))));
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(wallet, digest);
        assertEq(wallet.addr, precompiles.ecrecover_(digest, v, r, s));
    }

    function test_fuzzEdverify(bytes32 key, bytes memory preimage) public {
        // In the Rust code we don't do right truncation, so just assume:
        vm.assume(preimage.length % 32 == 0);
        (
            bytes32 digestA,
            bytes32 digestB,
            bytes32 pubKey,
            bytes32 sigA,
            bytes32 sigB
        ) = precompiles.createEd25519(key, preimage);
        vm.resetGasMetering();
        precompiles.testEd25519(digestA, digestB, pubKey, sigA, sigB);
    }

    function test_secp256r1(uint256 key, bytes memory preimage) public {
        (uint256 qx, uint256 qy) = vm.publicKeyP256(key);
        bytes32 d = keccak256(preimage);
        (bytes32 r, bytes32 s) = vm.signP256(key, d);
        vm.resetGasMetering();
        precompiles.secp256r1(d, r, s, bytes32(qx), bytes32(qy));
    }*/
}
