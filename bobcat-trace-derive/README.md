
# bobcat-trace-derive

bobcat-trace-derive is a proc-macro that annotates a function, printing every line it sees
to the Stylus console, when the `console` feature is set in the crate it's being used with:

```rust
#[unsafe(no_mangle)]
#[bobcat_trace]
pub unsafe extern "C" fn user_entrypoint(len: usize) -> usize {
    let args = read_args_vec(len);
    #[cfg(feature = "arbos")]
    let imm = Imm::default();
    #[cfg(not(feature = "arbos"))]
    let imm = Imm {
        should_server_sig_validate: true,
        should_user_sig_validate: true,
        server_key: SERVER_PUBLIC_KEY,
    };
    #[cfg(feature = "arbos")]
    assert_eq!(31337, chain_id(), "chain id not arbos-forge");
    flush_guard(|| entry(&imm, Args::deserialize(&mut args.as_slice()).unwrap()))
}
```

When this function is run and `console` can be used with Stylus:

```rust
Stylus says: [contract/src/main.rs:31] let args = read_args_vec(len);

Stylus says: [contract/src/main.rs:32] #[cfg(feature = "arbos")] let imm = Imm :: default();

Stylus says: [contract/src/main.rs:40] #[cfg(feature = "arbos")] assert_eq!
(31337, chain_id(), "chain id not arbos-forge");

Stylus says: [contract/src/main.rs:42] flush_guard(| |
entry(& imm, Args :: deserialize(& mut args.as_slice()).unwrap()))

Stylus says: [contract/src/main.rs:42] entry(& imm, Args :: deserialize(& mut args.as_slice()).unwrap())

Stylus says: panicked at contract/src/main.rs:42:72:
called `Result::unwrap()` on an `Err` value: Custom { kind: InvalidData, error: "Unexpected variant tag: 48" }
```
