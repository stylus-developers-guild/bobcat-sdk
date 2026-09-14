
# bobcat-cd-derive

`bobcat-cd-derive` is a proc macro for encoding and decoding EVM calldata. Functions are
available for encoding to a vector when `alloc` is enabled, and otherwise an API to encode
to a slice is accessible.

```rust
// This is a new type that has the maximum length of a String argument.
// This will be provided to any consumers in the form of a String using
// Into when they ask. We need to provide this information to our derive
// macro so that we may avoid alloc.
type DogName = EvmCdString<0, 100>;

// In dog Hotel California, dogs can arrive, but they can never leave. Of
// course, this is a great time for the dogs, and they love their lives
// here for good reason. It's an arf of a good time.
#[derive(EvmCdSerialise, EvmCdDeserialise)]
type DogsHotelCalifornia {
    DogsInHotel,
    EnrollDogInHotel(DogName),
    GiveDogTreat(DogName, DogTreat),
}

// Enum types are implicitly encoded as uint8, except if they exceed the
// maximum size we can tolerate here.
#[derive(EvmCdSerialise, EvmCdDeserialise)]
enum DogTreat {
    Biscuit,
    // Cheese is apparently a hallucinogen for dogs, and it brings them
    // to a state of ecstasy. Be careful not to feed the dogs too much.
    Cheese,
}

// I feel this is a simpler and richer way of engaging with Rust's
// built-in types, making a deeper pattern match simpler:

const SLOT_DOGS_IN_HOTEL: U = U::ZERO;

fn bump_dog_count() {
    storage_wrapping_add(&SLOT_DOGS_IN_HOTEL, &U::ONE)
}

#[unsafe(no_mangle)]
fn user_entrypoint(args_len: usize) -> usize {
    match read_args::<_>(args_len) {
        DogHotelCalifornia::DogsInHotel => write_result_word(&storage_load(&SLOT_DOGS_IN_HOTEL)),
        DogHotelCalifornia::EnrollDogInHotel(_) when n == "Boy".to_owned() => {
            bump_dog_count();
            write_result_str("Boy is so cute! Better rub that belly.")
        },
        DogHotelCalifornia::EnrollDogInHotel(_) when n == "Leo".to_owned() => {
            bump_dog_count();
            write_result_str("Leo is so sneaky! Be careful not to give him any food.")
        },
        DogHotelCalifornia::EnrollDogInHotel(_) => {
            bump_dog_count();
            write_result_str("I don't know this dog, but it's cute!"),
        }
        // I think you get the point here!
    }
}
```

