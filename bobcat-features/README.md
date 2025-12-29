
## bobcat-features

bobcat-features can be used for feature-driven development using a macro with
`bobcat-storage`:

```rust
bobcat_feature(internal_balances);
```

Would create a macro that accesses the storage slot for the feature
"`bobcat.features.internal_balances`".

```rust
IF_FEATURE_INTERNAL_BALANCES(
    {
        // This happens if the flag is turned on:
    },
    {
        // This happens if the flag is turned off:
    }
);
```
