# bobcat-cd-derive

Derive macros for `bobcat_cd::EvmCdSerialise` and
`bobcat_cd::EvmCdDeserialise`.

The derives support named, tuple, and unit structs, plus unit, tuple, and
struct enum variants. A top-level derived enum is an EVM function-call sum
type:

- the Rust variant name is converted to lower camel case;
- its field types provide their canonical Solidity ABI type names;
- the first four bytes of `keccak256("name(type,...)")` are written first;
- deserialisation reads those four bytes and uses them to select the variant;
- variant fields use Solidity ABI head/tail layout, including dynamic strings.

For example, `EnrollDogInHotel(EvmCdString<0, 100>)` uses the selector for
`enrollDogInHotel(string)`.

When an enum is itself used as a field, it is represented by its zero-based
variant index in a 32-byte `uint8` word. This is intended for fieldless
Solidity-style enums. An enum may contain at most 256 variants. Explicit Rust
discriminants and unions are rejected.

Currently inferred ABI names include:

- `u8`, `u16`, `u32`, `u64`, and `u128` as their corresponding `uintN`;
- `usize` as `uint32`;
- `bobcat_maths::U` as `uint256`;
- `[u8; N]` as `bytesN`;
- `Address` as `address` (left-padded to an ABI word);
- `Vec<u8>` as `bytes` when allocation support is enabled;
- `EvmCdArray<T, MIN, CAP>` as `T[]` without allocation;
- `EvmCdString<MIN, CAP>` as `string`;
- derived structs as Solidity tuple types;
- derived enums used as fields as `uint8`.

Because ABI names are supplied through trait methods, Rust type aliases work:
an alias such as `type DogName = EvmCdString<0, 100>` still contributes
`string` to the selector.

`EvmCdString<MIN, CAP>` stores UTF-8 bytes inline and therefore needs no heap
allocation. `MIN` and `CAP` are byte lengths. Its standalone representation is
a 32-byte offset word, a 32-byte length word, the UTF-8 bytes, and zero padding
to a 32-byte boundary. Within an enum variant's arguments it participates in
the normal ABI head/tail layout.

The `derive` feature is enabled by default in both `bobcat-cd` and
`bobcat-sdk`. When default features are disabled, enable it explicitly:

```toml
[dependencies]
bobcat-cd = { version = "0.9.8", features = ["derive"] }
```

```rust
use bobcat_cd::{EvmCdDeserialise, EvmCdSerialise, EvmCdString};

type DogName = EvmCdString<0, 100>;

#[derive(Debug, PartialEq, EvmCdSerialise, EvmCdDeserialise)]
enum DogTreat {
    Biscuit,
    Cheese,
}

#[derive(Debug, PartialEq, EvmCdSerialise, EvmCdDeserialise)]
enum DogsHotelCalifornia {
    DogsInHotel,
    EnrollDogInHotel(DogName),
    GiveDogTreat(DogName, DogTreat),
}

let command = DogsHotelCalifornia::EnrollDogInHotel(
    DogName::try_from("Boy").unwrap(),
);

// A fixed slice works as the writer, so encoding does not require alloc.
let mut storage = [0u8; 164];
let mut output = storage.as_mut_slice();
command.serialise(&mut output).unwrap();
let written = storage.len() - output.len();

let decoded = DogsHotelCalifornia::deserialise_reader(
    &mut &storage[..written],
).unwrap();
assert_eq!(decoded, command);
```

Derived structs concatenate their fields and do not add a selector of their
own. A selector belongs to an enum variant because the variant supplies the
function name and argument list.
