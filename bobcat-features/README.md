
## bobcat-features

bobcat-features can be used for feature-driven development using a macro with
`bobcat-storage`:

```rust
bobcat_feature!(internal_balances);
```

Would create a macro that accesses the storage slot for the feature
"`bobcat.features.internal_balances`" after taking -1 so as to take it off the curve, so
`uint256(abi.encodePacked("bobcat.features.internal_balances")) - 1`. If true is set, to
the feature flag, then the code associated executes.

```rust
IF_FEATURE_INTERNAL_BALANCES!({
    // This happens if the flag is turned on:
} else {
    // This happens if the flag is turned off:
});
```
