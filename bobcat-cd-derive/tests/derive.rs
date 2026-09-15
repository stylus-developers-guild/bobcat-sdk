use bobcat_cd::{
    const_keccak_sel,
    serialisation::{EvmCdDeserialise, EvmCdSerialise, EvmCdString},
};
use bobcat_cd_derive::{EvmCdDeserialise, EvmCdSerialise};

#[derive(Debug, PartialEq, Eq, EvmCdSerialise, EvmCdDeserialise)]
struct Named {
    small: u8,
    large: u32,
}

#[derive(Debug, PartialEq, Eq, EvmCdSerialise, EvmCdDeserialise)]
struct Tuple(u16, u8);

#[derive(Debug, PartialEq, Eq, EvmCdSerialise, EvmCdDeserialise)]
struct Unit;

#[derive(Debug, PartialEq, Eq, EvmCdSerialise, EvmCdDeserialise)]
enum Message {
    Ping,
    Tuple(u16, u8),
    Named { value: u32 },
}

type Name = EvmCdString<0, 32>;

#[derive(Debug, PartialEq, Eq, EvmCdSerialise, EvmCdDeserialise)]
#[evm_values]
pub enum Asset {
    USDC = 0,
    ARB = 1,
    WETH = 2,
}

#[derive(Debug, PartialEq, Eq, EvmCdSerialise, EvmCdDeserialise)]
#[evm_values]
enum SparseAsset {
    USDC = 3,
    ARB = 17,
    WETH = 255,
}

#[derive(Debug, PartialEq, Eq, EvmCdSerialise, EvmCdDeserialise)]
enum AssetCall {
    SetAsset(Asset),
}

#[derive(Debug, PartialEq, Eq, EvmCdSerialise, EvmCdDeserialise)]
enum Treat {
    Biscuit,
    Cheese,
}

#[derive(Debug, PartialEq, Eq, EvmCdSerialise, EvmCdDeserialise)]
enum DogCommand {
    DogsInHotel,
    EnrollDogInHotel(Name),
    GiveDogTreat(Name, Treat),
}

#[derive(Debug, PartialEq, Eq, EvmCdSerialise, EvmCdDeserialise)]
struct DogRecord {
    name: Name,
    treats: u8,
}

#[derive(Debug, PartialEq, Eq, EvmCdSerialise, EvmCdDeserialise)]
enum RecordCommand {
    Save(DogRecord),
    AwkwardNames { writer: u8, tail_offset: u8 },
    FixedBytes([u8; 4], u8),
}

#[derive(Debug, PartialEq, Eq, EvmCdSerialise, EvmCdDeserialise)]
struct Generic<T>
where
    T: PartialEq + Eq,
{
    value: T,
}

#[derive(Debug, PartialEq, Eq, EvmCdSerialise, EvmCdDeserialise)]
struct GenericNameCollision<__EvmCdWriter, __EvmCdReader> {
    writer: __EvmCdWriter,
    reader: __EvmCdReader,
}

#[derive(Debug, PartialEq, Eq, EvmCdSerialise, EvmCdDeserialise)]
enum SelectorCollision {
    XMLHttp(u8),
    XmlHttp(u8),
}

fn round_trip<T>(value: T)
where
    T: core::fmt::Debug + PartialEq + EvmCdSerialise + EvmCdDeserialise,
{
    let mut encoded = Vec::new();
    value.serialise(&mut encoded).unwrap();
    let decoded = T::deserialise_reader(&mut encoded.as_slice()).unwrap();
    assert_eq!(decoded, value);
}

#[test]
fn deserialises_from_arrays_slices_references_and_boxed_slices() {
    let value = Named {
        small: 7,
        large: 0x1234_5678,
    };
    let mut encoded = [0u8; 64];
    value.serialise(&mut encoded.as_mut_slice()).unwrap();

    fn from_array_of_any_size<T, const N: usize>(bytes: &[u8; N]) -> Result<T, std::io::Error>
    where
        T: EvmCdDeserialise,
    {
        T::deserialise(bytes)
    }

    assert_eq!(
        from_array_of_any_size::<Named, 64>(&encoded).unwrap(),
        value
    );
    assert!(from_array_of_any_size::<Named, 0>(&[]).is_err());

    let slice = encoded.as_slice();
    assert_eq!(Named::deserialise(slice).unwrap(), value);
    assert_eq!(Named::deserialise(&slice).unwrap(), value);

    let slice_reference = &slice;
    assert_eq!(Named::deserialise(slice_reference).unwrap(), value);

    let boxed: Box<[u8]> = encoded.into();
    assert_eq!(Named::deserialise(&boxed).unwrap(), value);
}

#[test]
fn derives_structs_in_field_order() {
    let value = Named {
        small: 7,
        large: 0x1234_5678,
    };
    let mut encoded = Vec::new();
    value.serialise(&mut encoded).unwrap();

    assert_eq!(encoded.len(), 64);
    assert_eq!(encoded[31], 7);
    assert_eq!(&encoded[60..64], &0x1234_5678u32.to_be_bytes());
    round_trip(value);
    round_trip(Tuple(0x1234, 9));
    round_trip(Unit);
}

#[test]
fn evm_values_enums_encode_uint8_values_without_selectors() {
    for (asset, discriminant) in [(Asset::USDC, 0), (Asset::ARB, 1), (Asset::WETH, 2)] {
        let mut encoded = Vec::new();
        asset.serialise(&mut encoded).unwrap();

        assert_eq!(encoded.len(), 32);
        assert!(encoded[..31].iter().all(|byte| *byte == 0));
        assert_eq!(encoded[31], discriminant);
        assert_eq!(Asset::deserialise(&encoded).unwrap(), asset);
    }

    for (asset, discriminant) in [
        (SparseAsset::USDC, 3),
        (SparseAsset::ARB, 17),
        (SparseAsset::WETH, 255),
    ] {
        let mut encoded = Vec::new();
        asset.serialise(&mut encoded).unwrap();
        assert_eq!(encoded[31], discriminant);
        assert_eq!(SparseAsset::deserialise(&encoded).unwrap(), asset);
    }

    let mut unknown = [0u8; 32];
    unknown[31] = 4;
    assert!(SparseAsset::deserialise(&unknown).is_err());
}

#[test]
fn evm_values_enums_remain_uint8_when_nested_in_calls() {
    let value = AssetCall::SetAsset(Asset::WETH);
    let mut encoded = Vec::new();
    value.serialise(&mut encoded).unwrap();

    assert_eq!(&encoded[..4], &const_keccak_sel(b"setAsset(uint8)"));
    assert_eq!(encoded.len(), 36);
    assert!(encoded[4..35].iter().all(|byte| *byte == 0));
    assert_eq!(encoded[35], 2);
    assert_eq!(AssetCall::deserialise(&encoded).unwrap(), value);
}

#[test]
fn derives_all_enum_variant_shapes() {
    for value in [
        Message::Ping,
        Message::Tuple(0x1234, 9),
        Message::Named { value: 0x1234_5678 },
    ] {
        round_trip(value);
    }

    let mut encoded = Vec::new();
    Message::Tuple(1, 2).serialise(&mut encoded).unwrap();
    assert_eq!(&encoded[..4], &const_keccak_sel(b"tuple(uint16,uint8)"));
}

#[test]
fn preserves_generics_and_where_clauses() {
    round_trip(Generic { value: 42u16 });
    round_trip(GenericNameCollision {
        writer: 7u8,
        reader: 9u16,
    });
}

#[test]
fn rejects_ambiguous_selector_sets() {
    let mut encoded = Vec::new();
    assert!(
        SelectorCollision::XMLHttp(1)
            .serialise(&mut encoded)
            .is_err()
    );
    assert!(
        SelectorCollision::XmlHttp(1)
            .serialise(&mut encoded)
            .is_err()
    );

    let selector = const_keccak_sel(b"xmlHttp(uint8)");
    assert!(SelectorCollision::deserialise_reader(&mut selector.as_slice()).is_err());
}

#[test]
fn rejects_unknown_enum_selectors() {
    let encoded = [0xff; 4];
    assert!(Message::deserialise_reader(&mut encoded.as_slice()).is_err());
}

#[test]
fn infers_solidity_selectors_from_variant_and_field_types() {
    let cases = [
        (DogCommand::DogsInHotel, const_keccak_sel(b"dogsInHotel()")),
        (
            DogCommand::EnrollDogInHotel(Name::try_from("Boy").unwrap()),
            const_keccak_sel(b"enrollDogInHotel(string)"),
        ),
        (
            DogCommand::GiveDogTreat(Name::try_from("Leo").unwrap(), Treat::Cheese),
            const_keccak_sel(b"giveDogTreat(string,uint8)"),
        ),
    ];

    for (value, selector) in cases {
        let mut encoded = Vec::new();
        value.serialise(&mut encoded).unwrap();
        assert_eq!(&encoded[..4], &selector);
        assert_eq!(
            DogCommand::deserialise_reader(&mut encoded.as_slice()).unwrap(),
            value,
        );
    }
}

#[test]
fn nested_enums_remain_uint8_values() {
    let value = DogCommand::GiveDogTreat(Name::try_from("Leo").unwrap(), Treat::Cheese);
    let mut encoded = Vec::new();
    value.serialise(&mut encoded).unwrap();

    assert_eq!(encoded.len(), 4 + 64 + 64);
    assert!(encoded[4..35].iter().all(|byte| *byte == 0));
    assert_eq!(encoded[35], 64, "string tail starts after the two heads");
    assert!(encoded[36..67].iter().all(|byte| *byte == 0));
    assert_eq!(encoded[67], 1, "nested enum is a uint8 ABI word");
    assert_eq!(encoded[99], 3, "dynamic string tail starts with its length");
    assert_eq!(&encoded[100..103], b"Leo");
}

#[test]
fn dynamic_struct_arguments_use_nested_tuple_offsets() {
    let value = RecordCommand::Save(DogRecord {
        name: Name::try_from("Boy").unwrap(),
        treats: 2,
    });
    let mut encoded = Vec::new();
    value.serialise(&mut encoded).unwrap();

    assert_eq!(&encoded[..4], &const_keccak_sel(b"save((string,uint8))"));
    assert_eq!(encoded[35], 32, "outer head points at tuple tail");
    assert_eq!(encoded[67], 64, "tuple head points at string tail");
    assert_eq!(encoded[99], 2);
    assert_eq!(encoded[131], 3);
    assert_eq!(&encoded[132..135], b"Boy");
    assert_eq!(
        RecordCommand::deserialise_reader(&mut encoded.as_slice()).unwrap(),
        value,
    );
}

#[test]
fn generated_locals_do_not_conflict_with_named_fields() {
    round_trip(RecordCommand::AwkwardNames {
        writer: 1,
        tail_offset: 2,
    });
}

#[test]
fn fixed_bytes_are_right_padded_to_an_abi_word() {
    let value = RecordCommand::FixedBytes(*b"woof", 7);
    let mut encoded = Vec::new();
    value.serialise(&mut encoded).unwrap();

    assert_eq!(
        &encoded[..4],
        &const_keccak_sel(b"fixedBytes(bytes4,uint8)")
    );
    assert_eq!(&encoded[4..8], b"woof");
    assert!(encoded[8..36].iter().all(|byte| *byte == 0));
    assert_eq!(encoded[67], 7);
    assert_eq!(
        RecordCommand::deserialise_reader(&mut encoded.as_slice()).unwrap(),
        value,
    );
}

#[test]
fn rejects_noncanonical_dynamic_offsets() {
    let value = RecordCommand::Save(DogRecord {
        name: Name::try_from("Boy").unwrap(),
        treats: 2,
    });
    let mut encoded = Vec::new();
    value.serialise(&mut encoded).unwrap();
    encoded[35] = 64;

    assert!(RecordCommand::deserialise_reader(&mut encoded.as_slice()).is_err());
}
