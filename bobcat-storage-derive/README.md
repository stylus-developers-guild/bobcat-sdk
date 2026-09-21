
# bobcat-storage-derive

bobcat-storage-derive is a derive macro for creating EVM storage accessors for a Rust
structure. It uses a phantom type based capabilities system to support programmers
building type-based restrictions for their storage accesses, and a trait to know how
to encode values to storage based on their size.

EvmBox is slightly misleading: it also functions as a RAII guard to know when to flush
storage access for std/host users.

```rust

pub struct Version<C: Cap> {
    pub no: EvmBox<C, u8>
}

// Storage access is prefixed with a "domain" specific to the storage
// structure being passed around.

#[derive(EvmStorage, Debug, Clone)]
#[evm_domain("superposition.storage")]
pub struct Storage<CVer: Cap, CDomain: Cap> {
    pub something: String,
    pub version: EvmBox<CVer, Version<CVer>>,
    pub domain: EvmBox<CDomain, String>,
    pub always_permissive: EvmBox<CA, u128>,
}

// This forms the basis of capabilities for the EVM's storage system.
// Using phantom types to type the capabilities of functions, it's
// possible to write functions that do things like this:

fn mutate_version<CW>(store: &mut Storage<CW, CA>) {
    // This function would be type restricted if we didn't have the
    // capability to manipulate this!
    store.version.no.set(100);
}

#[derive(EvmCdSerialise, EvmCdDeserialise, Debug, Clone)]
pub enum Entry {
    GetVersion,
    AccessControl
}

#[unsafe(no_mangle)]
fn user_entrypoint(len: usize) -> usize {
    let mut store = Storage::<CA, CA>();
    flush_guard(|| match read_cd::<_>(len) {
        Entry::GetVersion => store.version.into(),
        Entry::AccessControl => {
        // You could downgrade the powers of the app based on a
        // function in the storage here. TODO.
        }
    })
}
```
