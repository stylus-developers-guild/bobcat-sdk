
<img src="logo.svg" alt="bobcat-sdk logo" width="160"/>

# bobcat-sdk

bobcat-sdk is an opinionated SDK for Arbitrum Stylus, intended for advanced users.

## Codesize savings

bobcat-sdk is smaller than stylus-sdk. Check `examples` to see a comparison of a few
testing items, and a discussion of the methodology.

## Disclaimer

If you've never worked with Arbitrum Stylus before, this SDK is not for you. Check out the
official stylus-sdk repository first.

## Usage

Note: a generated storage selector is coming soon.

```rust
// main.rs

#![no_main]
#![no_std]

use bobcat_sdk::{
    cd::{const_keccak_sel, read_words},
    entry::*,
    maths::U,
    storage::*,
};

const SEL_NUMBER: [u8; 4] = const_keccak_sel(b"number()");
const SEL_SET_NUMBER: [u8; 4] = const_keccak_sel(b"setNumber(uint256)");
const SEL_MUL_NUMBER: [u8; 4] = const_keccak_sel(b"mulNumber(uint256)");
const SEL_ADD_NUMBER: [u8; 4] = const_keccak_sel(b"addNumber(uint256)");
const SEL_INCREMENT: [u8; 4] = const_keccak_sel(b"increment()");
const SEL_ADD_FROM_MSG_VALUE: [u8; 4] = const_keccak_sel(b"addFromMsgValue()");

#[unsafe(no_mangle)]
pub unsafe extern "C" fn user_entrypoint(args_len: usize) -> usize {
    let args = read_args_safe!(args_len, { 32 + 4 });
    let sel: [u8; 4] = args[..4].try_into().unwrap();
    let w = read_words!(&args[4..], 1);
    flush_guard(|| match sel {
        SEL_NUMBER => write_result_word(&storage_load(&U::ZERO)),
        SEL_SET_NUMBER => storage_store(&U::ZERO, w),
        SEL_MUL_NUMBER => storage_wrapping_mul(&U::ZERO, w),
        SEL_ADD_NUMBER => storage_wrapping_add(&U::ZERO, w),
        SEL_INCREMENT => storage_wrapping_add(&U::ONE, w),
        SEL_ADD_FROM_MSG_VALUE => storage_wrapping_add(&msg_value(), w),
        _ => unimplemented!(),
    });
    0
}

```

## Features

The feature flags below are the user-facing options for the `bobcat-sdk` crate. Workspace
crates also define internal/testing flags for examples and tooling; those are omitted
here intentionally.

- `std`: Enable standard library support across the bobcat subcrates that ship with the
SDK.

- `alloc`: Enable allocator-backed APIs (including the optional `bobcat-alloc`) across the
SDK crates that need heap support.

- `serde`: Add `serde` derives/traits for math types via `bobcat-maths`.

- `borsh`: Add `borsh` derives/traits for math types via `bobcat-maths`.

- `console`: Enable console logging via `bobcat-console` plus panic/host console hooks.

- `arbitrary`: Enable `arbitrary` support for math types (useful for fuzzing).

- `proptest`: Enable property-based testing helpers for maths and storage types.

- `alloy-enabled`: Enable integration with `alloy-primitives` for maths/entry types.

- `ruint-enabled`: Enable ruint-backed math helpers (used for mul/div paths).

- `panic`: Use the minimal panic handler that reverts via `unreachable`.

- `panic-revert`: Panic handler that reverts with a stack trace message.

- `panic-loc`: Panic handler that reports filename + line only.

- `panic-trace`: Panic handler that writes a trace identifier from transient storage into
the revert message.

- `msg-on-sdk-err`: Emit SDK error messages (overflow/decoding) when panicking to make
failures easier to diagnose.

- `detailed-errors`: Use detailed panic strings for SDK error helpers instead of generic
messages or codes.

- `dont-define-symbols`: Avoid defining the default `mark_used` symbol from `bobcat-entry`
when you need to provide your own entry symbols.

- `wasm-bindgen`: Enable `wasm-bindgen` integration for math types (wasm tooling).

- `wasm-bindgen-wasi`: Enable `wasm-bindgen` integration for WASI builds of the math
types.

- `ed25519-dalek`: Enable ed25519 support in the precompile helpers via `ed25519-dalek`.

- `shadow`: Enable the "shadow" event mechanism for custom chain event piping.

- `mutex`: Allow the storage implementation to use a mutex for multi-threaded access in
host-backed environments.


## Goals

1. Macro/compile time heavy features equivalent to the SDK.

2. Math operations, and native U256 and I256 types that are simple slices. Use the hidden
math operations in the Stylus VM for everything to keep codesize low.

3. Calling interface (CALL, STATICCALL, DELEGATECALL). Provide simple functions for
unpacking the results.

4. Creation (CREATE1/CREATE2) interface.

5. Minimal proxy creation features à la Vyper. Include static proxies, proxies that read
from a slot, and beacon proxies.

6. Each library feature can be imported as a separate package without the whole SDK.

7. Have to opt into the allocator. The wasm target is reexported from `mini-alloc`, though
the risc-v build is supplied here and based on the `risc-runner`.

8. Solidity storage equivalence.

9. Reentrancy guard using a Vyper-like exchange method of protecting the method id for
per-function reentrancy guards.

10. Support for every interface Vyper provides (these are the most common), plus a few
that the Superposition team makes frequent use of (Chainlink data feeds, Permit, LzRead,
Camelot, 9lives, Uniswap). We also welcome community contributions on this front.

11. The ability to distill built-in functions to basic types to reduce codesize if needed.
Most functions will ship with a Result and Option form, but they will also expose raw
values and simple booleans when that is helpful. Result and Option increase codesize (with
Option having a smaller impact), so developers focused on reducing their codesize can
choose the lighter forms.

12. Code verification using a bobcat-build script that adds a custom section with
information on the repo and the hash.

13. Support for [Markov Geist's on-chain riscv32im `risc-runner`](https://code.markovgeist.org) that
can run programs in a syscall pledge(2)-style jail.

## Non-goals

1. Test-mocking features (such as setting the sender). Use ArbOS-Foundry for end-to-end
testing instead. These functions will be provided, but you cannot set them. Check
`e2e-test` for a practical example.

2. Solidity code generation of the ABI.

3. Entrypoint code generation. Function selection is okay.

4. Storage of integers other than U256. Conversion is okay.

6. Visibility setting on the functions.

7. Payable designations for functions.

8. Trait inheritance.

9. Code that performs high-level execution of interface calldata. For example, code that
calls ERC-20 on other tokens for you using `static_call`. In our experience, you will
define a per-application mock for the native host for these calls. The mock usually
resembles thread-local storage for balances. We avoid generating code like that because we
find the approach in the OZ repo too cumbersome for our (Superposition) use cases. We
encourage users to rely on arbos-foundry for end-to-end mocking while providing the basics
to compile the program for everything else. We still provide decoding functions for the
interfaces.

## Maths

With wasm, for codesize, it is best to use either 32-bit numbers or go all-in with the
entire 256-bit native EVM number. The machine is natively 32-bit (though it has operations
for 64 bit numbers), so operations involving those values do not require additional code
generation. T Stylus machine provides operations for 256-bit math that we can use to keep
codesize down. A good reason to use other integer types is to keep calldata low when you
encode with a different format.

The native integer types in this SDK use Stylus functions for math whenever possible,
keeping codesize (and gas, we imagine) very low. Some functions we use frequently in web3
are also included, such as `mul_div`, `mul_div_round_up`, and several widening operations.
These functions use the native VM operations for minimal codesize impact.

## Constant functions

Some functions are available in a const form. These are implemented in code, avoiding the
host features. Do not use the const variants at runtime. Prefer the non-const functions to
reduce the amount of codesize in the generated code by leaning on the host implementations.

It is fine to use the constant functions inside a function that performs the calculation,
like so:

```rust
pub fn get_ed25519_count() -> U {
    storage_load(const_slot_off_curve("superposition.passport.ed25519_count"))
}
```

## Philosophy

This SDK strives to be like the bobcat: nimble, stalking its prey in winter, and
conserving its energy. It is tiny, focused, and conservative, with the ability to opt out
of parts of the library. Once this SDK is finished, there will be no new features except
for adopting new wasm features or supporting ArbOS upgrades. If you want shiny new things,
use the mainstream SDK or add the features yourself.

## Why make this?

At Superposition, we often run up against codesize restrictions. We are opinionated with
our development practices, including developing contracts that use a Solana-style decoding
method instead of the classic EVM calldata format. The feature-rich SDK hinders our
development practices when we go off the beaten path (which happens a lot). We are also
frequently victims of code rot in the main SDK. We wanted something small and versatile
that lets us lean on ArbOS-Foundry for end-to-end contract testing, with only the features
we truly need.

## Quick start with `bobcat-new`

Use `bobcat-new.sh` to scaffold a project quickly:

```sh
./bobcat-new.sh my-new-project
cd my-new-project
make my-new-project.wasm
```

The script creates a ready-to-build workspace with sensible defaults, a deploy script, and
a `Makefile` that produces the optimized wasm artifact.

## Deployment with `bobcat-deploy`

Use `bobcat-deploy` to do a simple deploy:

```sh
./bobcat-deploy <endpoint> <private key> <wasm file>
```

## Credits

Credit to Marko (Blazeaster) for the logo!

Bernard Wagner from iosiro has been an invaluable resource for developing this SDK. Thanks
to his work with arbos-foundry, and for his hands on support, this wouldn't have been
possible!
