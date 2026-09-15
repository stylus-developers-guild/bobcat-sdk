use bobcat_cd::serialisation::{EvmCdDeserialise, EvmCdSerialise, EvmCdString};

#[test]
fn fixed_capacity_string_round_trips_without_allocating() {
    type DogName = EvmCdString<0, 100>;

    let name = DogName::try_from("Boy").unwrap();
    let mut encoded = Vec::new();
    name.serialise(&mut encoded).unwrap();

    assert_eq!(encoded.len(), 96);
    assert_eq!(encoded[31], 32);
    assert_eq!(encoded[63], 3);
    assert_eq!(&encoded[64..67], b"Boy");
    assert!(encoded[67..].iter().all(|byte| *byte == 0));

    let decoded = DogName::deserialise_reader(&mut encoded.as_slice()).unwrap();
    assert_eq!(decoded.as_str(), "Boy");
}

#[test]
fn fixed_capacity_string_enforces_utf8_byte_bounds() {
    type DogName = EvmCdString<1, 4>;

    assert!(DogName::try_from("").is_err());
    assert_eq!(DogName::try_from("éé").unwrap().as_str(), "éé");
    assert!(DogName::try_from("ééx").is_err());
}

#[test]
fn fixed_capacity_string_rejects_invalid_input() {
    type DogName = EvmCdString<0, 4>;

    let mut too_long = vec![0u8; 96];
    too_long[31] = 32;
    too_long[63] = 5;
    assert!(DogName::deserialise_reader(&mut too_long.as_slice()).is_err());

    let mut invalid_utf8 = vec![0u8; 96];
    invalid_utf8[31] = 32;
    invalid_utf8[63] = 1;
    invalid_utf8[64] = 0xff;
    assert!(DogName::deserialise_reader(&mut invalid_utf8.as_slice()).is_err());
}
