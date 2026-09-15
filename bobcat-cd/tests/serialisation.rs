use bobcat_cd::serialisation::SelectorHasher;
use bobcat_cd::{
    EvmCdAddress, EvmCdArray, EvmCdArrayError, EvmCdDeserialise, EvmCdSerialise, EvmCdString,
    const_keccak_sel,
};

#[derive(Debug, PartialEq, Eq, EvmCdSerialise, EvmCdDeserialise)]
enum Call {
    Store(EvmCdAddress, EvmCdArray<u16, 0, 3>),
    SetCount(usize),
}

#[test]
fn selector_hasher_supports_signatures_larger_than_512_bytes() {
    let signature = [b'a'; 600];
    let selector = SelectorHasher::new().update(&signature).selector();

    assert_eq!(selector, const_keccak_sel(&signature));
}

#[test]
fn usize_uses_uint32_abi_encoding() {
    let call = Call::SetCount(u32::MAX as usize);
    let mut encoded = Vec::new();
    call.serialise(&mut encoded).unwrap();

    assert_eq!(&encoded[..4], &const_keccak_sel(b"setCount(uint32)"));
    assert!(encoded[4..32].iter().all(|byte| *byte == 0));
    assert_eq!(&encoded[32..36], &u32::MAX.to_be_bytes());
    assert_eq!(
        Call::deserialise_reader(&mut encoded.as_slice()).unwrap(),
        call
    );

    let mut noncanonical = encoded;
    noncanonical[4] = 1;
    assert!(Call::deserialise_reader(&mut noncanonical.as_slice()).is_err());

    if usize::BITS > 32 {
        let mut output = Vec::new();
        assert!(
            Call::SetCount(u32::MAX as usize + 1)
                .serialise(&mut output)
                .is_err()
        );
    }
}

#[test]
fn address_is_left_padded_and_uses_the_address_abi_type() {
    let address = EvmCdAddress::from([0xabu8; 20]);
    let values = EvmCdArray::try_from_array([7, 9, 0], 2).unwrap();
    let call = Call::Store(address, values);
    let mut encoded = Vec::new();
    call.serialise(&mut encoded).unwrap();

    assert_eq!(&encoded[..4], &const_keccak_sel(b"store(address,uint16[])"));
    assert!(encoded[4..16].iter().all(|byte| *byte == 0));
    assert_eq!(&encoded[16..36], &[0xab; 20]);
    assert_eq!(
        encoded[67], 64,
        "array tail starts after both argument heads"
    );
    assert_eq!(encoded[99], 2);
    assert_eq!(encoded[131], 7);
    assert_eq!(encoded[163], 9);
    assert_eq!(
        Call::deserialise_reader(&mut encoded.as_slice()).unwrap(),
        call
    );
}

#[test]
fn fixed_capacity_dynamic_array_enforces_bounds_without_allocating() {
    type Values = EvmCdArray<u16, 1, 3>;

    assert_eq!(
        Values::try_from_array([0, 0, 0], 0),
        Err(EvmCdArrayError::TooShort),
    );
    assert_eq!(
        Values::try_from_array([0, 0, 0], 4),
        Err(EvmCdArrayError::TooLong),
    );
    assert_eq!(
        EvmCdArray::<u16, 4, 3>::try_from_array([0, 0, 0], 3),
        Err(EvmCdArrayError::InvalidBounds),
    );

    let values = Values::try_from_slice(&[3, 5]).unwrap();
    assert_eq!(values.as_slice(), &[3, 5]);
    assert_eq!(values.len(), 2);
    assert_eq!(values.capacity(), 3);
}

#[test]
fn fixed_capacity_array_supports_dynamic_elements() {
    type Text = EvmCdString<0, 4>;
    type Texts = EvmCdArray<Text, 0, 2>;

    let values = Texts::try_from_array(
        [Text::try_from("a").unwrap(), Text::try_from("bc").unwrap()],
        2,
    )
    .unwrap();
    let mut encoded = Vec::new();
    values.serialise(&mut encoded).unwrap();

    assert_eq!(encoded.len(), 256);
    assert_eq!(encoded[31], 32, "standalone array head points at its tail");
    assert_eq!(encoded[63], 2);
    assert_eq!(
        encoded[95], 64,
        "first string offset follows both element heads"
    );
    assert_eq!(encoded[127], 128);
    assert_eq!(encoded[159], 1);
    assert_eq!(encoded[160], b'a');
    assert_eq!(encoded[223], 2);
    assert_eq!(&encoded[224..226], b"bc");

    let decoded = Texts::deserialise_reader(&mut encoded.as_slice()).unwrap();
    assert_eq!(decoded, values);
}
