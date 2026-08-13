#!/bin/sh -e

usage() {
	>&2 echo $@
	exit 2
}

project_name="$1"

[ -z "$project_name" ] && usage $(basename $0) "<project/folder name>"

case "$project_name" in
	*_*) usage "_ disallowed from project name. Use hyphens" ;;
	*[[:space:]]*) usage "spaces disallowed from project name" ;;
esac

project_name="$(echo "$project_name" | sed 's/\/*$//g')"

if [ -d "$project_name" ]; then
	>&2 echo "$project_name already exists! Aborting."
	exit 1
fi

lib_project_name="$(echo "$project_name" | sed 's/-/_/g')"
upper_project_name="$(echo "$project_name" |  sed 's/./\U&/; s/-\(.*\)/\U\1/')"

mkdir -p "$project_name/.cargo"

cd "$project_name"

mkdir src forge-lib test

cat >Cargo.toml <<EOF
[package]
name = "$project_name"
version = "0.1.0"
edition = "2024"

[lib]
name = "$lib_project_name"
crate-type   = ["rlib", "cdylib"]

[dependencies]
bobcat-sdk = { version = "0.6.6", features = ["panic"] }

[profile.release]
codegen-units = 1
opt-level = "z"
panic = "abort"
strip = true
lto = "fat"
debug = false
rpath = false
debug-assertions = false
incremental = false

[profile.dev]
codegen-units = 16
panic = "unwind"
opt-level = "z"
incremental = true

[features]
std = ["bobcat-sdk/std"]
EOF

cat >.cargo/config.toml <<EOF
[target.wasm32-unknown-unknown]
rustflags = [
  "-C", "target-feature=-reference-types",
  "-C", "target-feature=-multivalue"
]
EOF

cat >wasm-post.sh <<EOF
#!/bin/sh -eu

# We make a pass similar to the way the node operates (it does a
# wasm2wat->wat2wasm pass). We also do this for arbos-foundry.

f="\$(mktemp)"

wasm-opt \\
	--dce \\
	--rse \\
	--signature-pruning \\
	--strip-debug \\
	--enable-bulk-memory \\
	--strip \\
	-Oz \\
	"\$1" \\
	-o "\$f.wasm1"

wasm2wat -o \$f.wat \$f.wasm1

wat2wasm -o $project_name.wasm \$f.wat
EOF

chmod +x wasm-post.sh

cat >Makefile <<EOF

$project_name.wasm: \$(shell find Cargo.* src -type f)
	@rm -f $lib_project_name.wasm
	@cargo build --release --target wasm32-unknown-unknown
	@./wasm-post.sh \\
		target/wasm32-unknown-unknown/release/$project_name.wasm \\
		$lib_project_name.wasm
	@./check-codesize.sh $project_name.wasm
EOF

cat >check-codesize.sh <<EOF
#!/bin/sh

size="\$(du "\$1" | cut -f1)"

if [ "\$size" -gt 60 ]; then
	>& echo "\$1 too large"
	exit 1
fi

exit 0
EOF

chmod +x check-codesize.sh

cat >"src/lib.rs" <<EOF
#![cfg_attr(not(feature = "std"), no_std)]

use bobcat_sdk::{
    cd::const_keccak_sel,
    entry::{read_args_safe, write_result_slice},
    maths::U,
};

const SEL_HELLO: [u8; 4] = const_keccak_sel(b"hello()");

#[unsafe(no_mangle)]
pub unsafe extern "C" fn user_entrypoint(args_len: usize) -> usize {
    let args = &read_args_safe!(args_len, { 32 + 4 });
    let sel: [u8; 4] = args[..4].try_into().unwrap();
    match sel {
        SEL_HELLO => {
            write_result_slice(&U::from(123u32).0);
            0
        }
        _ => 1,
    }
}
EOF

cat >.gitignore <<EOF
$project_name.wasm
*.wat
*.wasm1
target
out
EOF

cat >rust-toolchain.toml <<EOF
[toolchain]
channel = "stable"
components = [ "rust-src" ]
EOF

cat >foundry.toml <<EOF
[profile.default]
src = "src"
out = "out"
libs = ["forge-lib"]
fs_permissions = [{ access = "read", path = "$lib_project_name.wasm"}]
EOF

cat >"test/IArbFoundry.sol" <<EOF
// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.20;

interface IArbFoundry {
    function deployStylusCode(string calldata artifactPath) external returns (address deployedAddress);
    function deployStylusCode(string calldata artifactPath, bytes calldata constructorArgs) external returns (address deployedAddress);
    function deployStylusCode(string calldata artifactPath, uint256 value) external returns (address deployedAddress);
    function deployStylusCode(string calldata artifactPath, bytes calldata constructorArgs, uint256 value) external returns (address deployedAddress);
    function deployStylusCode(string calldata artifactPath, bytes32 salt) external returns (address deployedAddress);
    function deployStylusCode(string calldata artifactPath, bytes calldata constructorArgs, bytes32 salt) external returns (address deployedAddress);
    function deployStylusCode(string calldata artifactPath, uint256 value, bytes32 salt) external returns (address deployedAddress);
    function deployStylusCode(string calldata artifactPath, bytes calldata constructorArgs, uint256 value, bytes32 salt) external returns (address deployedAddress);
}
EOF

cat >"src/I${upper_project_name}.sol" <<EOF
// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.20;

interface I${upper_project_name} {
    function hello() external pure returns (uint256);
}
EOF

cat >"test/$upper_project_name.t.sol" <<EOF
// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.20;

import {Test} from "forge-std/Test.sol";

import {IArbFoundry} from "./IArbFoundry.sol";

import {I$upper_project_name} from "../src/I${upper_project_name}.sol";

contract $upper_project_name is Test {
    I${upper_project_name} c;

    function setUp() external {
        c = I${upper_project_name}(IArbFoundry(address(vm)).deployStylusCode(
            "$lib_project_name.wasm"
        ));
    }

    function test_contractDeployed() public view {
        assertEq(123, c.hello());
    }
}
EOF

cat >tests.sh <<EOF
#!/bin/sh -e

# This is left here for you to comment out if you have functions you
# want to test in Rust on the native host:
#cargo test --features std

make

arbos-forge test \$@
EOF

chmod +x tests.sh

cat >README.md <<EOF

# $project-name

## Dependencies

1. (https://github.com/OffchainLabs/cargo-stylus)[\`cargo-stylus-sdk\`] -- Cargo Stylus
binary for deployment.

2. (https://github.com/iosiro/arbos-foundry)[\`arbos-foundry\`] -- Needed for testing.

3. Rust with wasm32-unknown-unknown.

## Building

	make

## Testing

	./tests.sh
EOF

git init

git submodule add --quiet https://github.com/foundry-rs/forge-std forge-lib/forge-std

if ! [ -z "$EDITOR" ]; then
	$EDITOR "src/lib.rs" &
fi
