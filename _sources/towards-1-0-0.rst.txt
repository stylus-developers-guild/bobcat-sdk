
"""""""""""""
Towards 1.0.0
"""""""""""""

Superposition is evolving Bobcat-sdk from a Superposition-only internal project to
something for everyone, with a book (and examples for every mainstream interface), a
derived capabilities-based storage macro, a stack-free overhauled calling interface,
on-chain verification, generation of Solidity code from the entrypoint types, and an easy
getting started url.

What is Bobcat-sdk?
-------------------

Bobcat-sdk is a simple to use and very codesize efficient SDK for Arbitrum Stylus, with
several features over the existing SDK:

1. Full traces on-chain from a panic macro.

2. Interfaces built into the interface for

```rust
pub struct Entry {
    Hello,
}
```

History of Bobcat-sdk
---------------------

We created the SDK out of a need for better codesize for our second flagship app, 9lives.
9lives started out as a permissionless prediction market, and evolved into a 0days option
platform. Stylus SDK at the time underwent a shift, adding abstractions to make testing
easier. But we were not able to migrate our code, because at the time, the more
abstractions

What's changed/why do this?
---------------------------

Arbos-forge is excellent and eliminates most of the pain points of developing with Stylus
once real world balances/calling is needed.
