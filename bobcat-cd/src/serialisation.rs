
use bobcat_maths::U;

use bobcat_storage::{Keccak256, keccak256_builder};

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
use alloc::{string::String, vec::Vec};

#[cfg(not(feature = "std"))]
mod no_std {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Error {
        WriteAllEof,
        ReadExactEof,
        InvalidData,
    }

    pub trait Write {
        fn write(&mut self, buf: &[u8]) -> Result<usize, Error>;
        fn flush(&mut self) -> Result<(), Error>;
        fn is_empty(&self) -> bool;

        fn write_all(&mut self, mut buf: &[u8]) -> Result<(), Error> {
            while !buf.is_empty() {
                match self.write(buf) {
                    Ok(0) => return Err(Error::WriteAllEof),
                    Ok(n) if n <= buf.len() => buf = &buf[n..],
                    Ok(_) => return Err(Error::InvalidData),
                    Err(error) => return Err(error),
                }
            }
            Ok(())
        }
    }

    pub trait Read {
        fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error>;

        fn read_exact(&mut self, mut buf: &mut [u8]) -> Result<(), Error> {
            while !buf.is_empty() {
                match self.read(buf) {
                    Ok(0) => break,
                    Ok(n) if n <= buf.len() => buf = &mut buf[n..],
                    Ok(_) => return Err(Error::InvalidData),
                    Err(error) => return Err(error),
                }
            }
            if buf.is_empty() {
                Ok(())
            } else {
                Err(Error::ReadExactEof)
            }
        }
    }

    impl Write for &mut [u8] {
        fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
            let len = core::cmp::min(self.len(), buf.len());
            let target = core::mem::take(self);
            let (written, remaining) = target.split_at_mut(len);
            written.copy_from_slice(&buf[..len]);
            *self = remaining;
            Ok(len)
        }

        fn flush(&mut self) -> Result<(), Error> {
            Ok(())
        }

        fn is_empty(&self) -> bool {
            <[u8]>::is_empty(self)
        }
    }

    impl Read for &[u8] {
        fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
            let len = core::cmp::min(self.len(), buf.len());
            buf[..len].copy_from_slice(&self[..len]);
            *self = &self[len..];
            Ok(len)
        }
    }
}

#[cfg(not(feature = "std"))]
pub use no_std::{Error, Read, Write};

#[cfg(feature = "std")]
pub use std::io::{Error, Read, Write};

#[doc(hidden)]
#[derive(Clone)]
pub struct SelectorHasher(Keccak256);

impl SelectorHasher {
    pub fn new() -> Self {
        Self(keccak256_builder())
    }

    pub fn update(self, bytes: &[u8]) -> Self {
        Self(self.0.update(bytes))
    }

    pub fn update_usize(self, mut value: usize) -> Self {
        let mut digits = [0u8; 20];
        let mut start = digits.len();
        loop {
            start -= 1;
            digits[start] = b'0' + (value % 10) as u8;
            value /= 10;
            if value == 0 {
                break;
            }
        }
        self.update(&digits[start..])
    }

    pub fn selector(&self) -> [u8; 4] {
        let hash = self.0.finalize();
        [hash[0], hash[1], hash[2], hash[3]]
    }
}

impl Default for SelectorHasher {
    fn default() -> Self {
        Self::new()
    }
}

pub trait EvmCdSerialise {
    fn serialise<W: Write>(&self, writer: &mut W) -> Result<(), Error>;

    #[doc(hidden)]
    fn serialise_value<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        self.serialise(writer)
    }

    #[doc(hidden)]
    fn is_abi_dynamic() -> bool {
        false
    }

    #[doc(hidden)]
    fn abi_head_size() -> usize {
        32
    }

    #[doc(hidden)]
    fn abi_tail_size(&self) -> usize {
        0
    }

    #[doc(hidden)]
    fn serialise_abi_head<W: Write>(
        &self,
        _tail_offset: usize,
        writer: &mut W,
    ) -> Result<(), Error> {
        self.serialise_value(writer)
    }

    #[doc(hidden)]
    fn serialise_abi_tail<W: Write>(&self, _writer: &mut W) -> Result<(), Error> {
        Ok(())
    }

    #[doc(hidden)]
    fn append_abi_type(hasher: SelectorHasher) -> SelectorHasher;
}

#[doc(hidden)]
pub enum EvmCdHead<T> {
    Value(T),
    Offset(usize),
}

pub trait EvmCdDeserialise: Sized {
    fn deserialise<B>(bytes: &B) -> Result<Self, Error>
    where
        B: AsRef<[u8]> + ?Sized,
    {
        let mut reader = bytes.as_ref();
        Self::deserialise_reader(&mut reader)
    }

    fn deserialise_reader<R: Read>(reader: &mut R) -> Result<Self, Error>;

    #[doc(hidden)]
    fn deserialise_value<R: Read>(reader: &mut R) -> Result<Self, Error> {
        Self::deserialise_reader(reader)
    }

    #[doc(hidden)]
    fn is_abi_dynamic() -> bool {
        false
    }

    #[doc(hidden)]
    fn abi_head_size() -> usize {
        32
    }

    #[doc(hidden)]
    fn abi_tail_size(&self) -> usize {
        0
    }

    #[doc(hidden)]
    fn deserialise_abi_head<R: Read>(reader: &mut R) -> Result<EvmCdHead<Self>, Error> {
        Ok(EvmCdHead::Value(Self::deserialise_value(reader)?))
    }

    #[doc(hidden)]
    fn deserialise_abi_finish<R: Read>(
        head: EvmCdHead<Self>,
        _expected_tail_offset: usize,
        _reader: &mut R,
    ) -> Result<Self, Error> {
        match head {
            EvmCdHead::Value(value) => Ok(value),
            EvmCdHead::Offset(_) => Err(invalid_data()),
        }
    }

    #[doc(hidden)]
    fn append_abi_type(hasher: SelectorHasher) -> SelectorHasher;
}

#[cfg(not(feature = "std"))]
pub fn invalid_data() -> Error {
    Error::InvalidData
}

#[cfg(feature = "std")]
pub fn invalid_data() -> Error {
    Error::new(std::io::ErrorKind::InvalidData, "invalid EVM calldata")
}

fn read_usize_word<R: Read>(reader: &mut R) -> Result<usize, Error> {
    let mut word = [0u8; 32];
    reader.read_exact(&mut word)?;
    if word[..32 - size_of::<usize>()]
        .iter()
        .any(|byte| *byte != 0)
    {
        return Err(invalid_data());
    }
    Ok(usize::from_be_bytes(
        word[32 - size_of::<usize>()..].try_into().unwrap(),
    ))
}

fn write_dynamic_tail<W: Write>(bytes: &[u8], writer: &mut W) -> Result<(), Error> {
    writer.write_all(&U::from_usize(bytes.len()).0)?;
    writer.write_all(bytes)?;
    const ZEROES: [u8; 31] = [0; 31];
    let padding = (32 - bytes.len() % 32) % 32;
    writer.write_all(&ZEROES[..padding])
}

fn write_dynamic_bytes<W: Write>(bytes: &[u8], writer: &mut W) -> Result<(), Error> {
    writer.write_all(&U::from_u32(32).0)?;
    write_dynamic_tail(bytes, writer)
}

fn dynamic_tail_size(len: usize) -> usize {
    32 + len + (32 - len % 32) % 32
}

fn read_dynamic_tail<const CAP: usize, R: Read>(
    reader: &mut R,
) -> Result<([u8; CAP], usize), Error> {
    let len = read_usize_word(reader)?;
    if len > CAP {
        return Err(invalid_data());
    }
    let mut bytes = [0u8; CAP];
    reader.read_exact(&mut bytes[..len])?;
    let padding = (32 - len % 32) % 32;
    let mut padding_bytes = [0u8; 31];
    reader.read_exact(&mut padding_bytes[..padding])?;
    if padding_bytes[..padding].iter().any(|byte| *byte != 0) {
        return Err(invalid_data());
    }
    Ok((bytes, len))
}

fn read_dynamic_bytes<const CAP: usize, R: Read>(
    reader: &mut R,
) -> Result<([u8; CAP], usize), Error> {
    if read_usize_word(reader)? != 32 {
        return Err(invalid_data());
    }
    read_dynamic_tail(reader)
}

impl EvmCdSerialise for U {
    fn serialise<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        writer.write_all(&self.0)
    }

    fn append_abi_type(hasher: SelectorHasher) -> SelectorHasher {
        hasher.update(b"uint256")
    }
}

impl EvmCdDeserialise for U {
    fn deserialise_reader<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let mut buf = [0u8; 32];
        reader.read_exact(&mut buf)?;
        Ok(U(buf))
    }

    fn append_abi_type(hasher: SelectorHasher) -> SelectorHasher {
        hasher.update(b"uint256")
    }
}

macro_rules! for_ints {
    ($($ty:ty => $abi:literal),+ $(,)?) => {
        $(
            impl EvmCdSerialise for $ty {
                fn serialise<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
                    writer.write_all(&[0u8; 32 - size_of::<$ty>()])?;
                    writer.write_all(&self.to_be_bytes())
                }

                fn append_abi_type(hasher: SelectorHasher) -> SelectorHasher {
                    hasher.update($abi)
                }
            }

            impl EvmCdDeserialise for $ty {
                fn deserialise_reader<R: Read>(reader: &mut R) -> Result<Self, Error> {
                    let U(word) = U::deserialise_reader(reader)?;
                    if word[..32 - size_of::<$ty>()].iter().any(|byte| *byte != 0) {
                        return Err(invalid_data());
                    }
                    Ok(<$ty>::from_be_bytes(
                        word[32 - size_of::<$ty>()..].try_into().unwrap(),
                    ))
                }

                fn append_abi_type(hasher: SelectorHasher) -> SelectorHasher {
                    hasher.update($abi)
                }
            }
        )+
    };
}

for_ints! {
    u8 => b"uint8",
    u16 => b"uint16",
    u32 => b"uint32",
    u64 => b"uint64",
    u128 => b"uint128",
}

impl EvmCdSerialise for usize {
    fn serialise<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        u32::try_from(*self)
            .map_err(|_| invalid_data())?
            .serialise(writer)
    }

    fn append_abi_type(hasher: SelectorHasher) -> SelectorHasher {
        hasher.update(b"uint32")
    }
}

impl EvmCdDeserialise for usize {
    fn deserialise_reader<R: Read>(reader: &mut R) -> Result<Self, Error> {
        Ok(u32::deserialise_reader(reader)? as usize)
    }

    fn append_abi_type(hasher: SelectorHasher) -> SelectorHasher {
        hasher.update(b"uint32")
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EvmCdAddress([u8; 20]);

impl EvmCdAddress {
    pub const fn new(bytes: [u8; 20]) -> Self {
        Self(bytes)
    }

    pub const fn into_array(self) -> [u8; 20] {
        self.0
    }

    pub const fn as_array(&self) -> &[u8; 20] {
        &self.0
    }
}

impl From<[u8; 20]> for EvmCdAddress {
    fn from(bytes: [u8; 20]) -> Self {
        Self::new(bytes)
    }
}

impl From<EvmCdAddress> for [u8; 20] {
    fn from(address: EvmCdAddress) -> Self {
        address.into_array()
    }
}

impl AsRef<[u8; 20]> for EvmCdAddress {
    fn as_ref(&self) -> &[u8; 20] {
        self.as_array()
    }
}

impl AsRef<[u8]> for EvmCdAddress {
    fn as_ref(&self) -> &[u8] {
        self.as_array()
    }
}

impl EvmCdSerialise for EvmCdAddress {
    fn serialise<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        writer.write_all(&[0; 12])?;
        writer.write_all(&self.0)
    }

    fn append_abi_type(hasher: SelectorHasher) -> SelectorHasher {
        hasher.update(b"address")
    }
}

impl EvmCdDeserialise for EvmCdAddress {
    fn deserialise_reader<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let mut word = [0u8; 32];
        reader.read_exact(&mut word)?;
        if word[..12].iter().any(|byte| *byte != 0) {
            return Err(invalid_data());
        }
        Ok(Self(word[12..].try_into().unwrap()))
    }

    fn append_abi_type(hasher: SelectorHasher) -> SelectorHasher {
        hasher.update(b"address")
    }
}

impl<const N: usize> EvmCdSerialise for [u8; N] {
    fn serialise<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        if N == 0 || N > 32 {
            return Err(invalid_data());
        }
        writer.write_all(self)?;
        const ZEROES: [u8; 32] = [0; 32];
        writer.write_all(&ZEROES[..32 - N])
    }

    fn append_abi_type(hasher: SelectorHasher) -> SelectorHasher {
        hasher.update(b"bytes").update_usize(N)
    }
}

impl<const N: usize> EvmCdDeserialise for [u8; N] {
    fn deserialise_reader<R: Read>(reader: &mut R) -> Result<Self, Error> {
        if N == 0 || N > 32 {
            return Err(invalid_data());
        }
        let mut word = [0u8; 32];
        reader.read_exact(&mut word)?;
        if word[N..].iter().any(|byte| *byte != 0) {
            return Err(invalid_data());
        }
        Ok(word[..N].try_into().unwrap())
    }

    fn append_abi_type(hasher: SelectorHasher) -> SelectorHasher {
        hasher.update(b"bytes").update_usize(N)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvmCdArrayError {
    InvalidBounds,
    TooShort,
    TooLong,
}

pub struct EvmCdArray<T, const MIN: usize, const CAP: usize> {
    len: usize,
    values: [core::mem::MaybeUninit<T>; CAP],
}

impl<T, const MIN: usize, const CAP: usize> EvmCdArray<T, MIN, CAP> {
    fn empty() -> Self {
        Self {
            len: 0,
            values: [const { core::mem::MaybeUninit::uninit() }; CAP],
        }
    }

    fn validate_len(len: usize) -> Result<(), EvmCdArrayError> {
        if MIN > CAP {
            return Err(EvmCdArrayError::InvalidBounds);
        }
        if len < MIN {
            return Err(EvmCdArrayError::TooShort);
        }
        if len > CAP {
            return Err(EvmCdArrayError::TooLong);
        }
        Ok(())
    }

    fn push(&mut self, value: T) {
        debug_assert!(self.len < CAP);
        self.values[self.len].write(value);
        self.len += 1;
    }

    pub fn try_from_array(values: [T; CAP], len: usize) -> Result<Self, EvmCdArrayError> {
        Self::validate_len(len)?;
        let mut out = Self::empty();
        for value in values.into_iter().take(len) {
            out.push(value);
        }
        Ok(out)
    }

    pub fn try_from_slice(values: &[T]) -> Result<Self, EvmCdArrayError>
    where
        T: Clone,
    {
        Self::validate_len(values.len())?;
        let mut out = Self::empty();
        for value in values {
            out.push(value.clone());
        }
        Ok(out)
    }

    pub const fn len(&self) -> usize {
        self.len
    }

    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub const fn capacity(&self) -> usize {
        CAP
    }

    pub fn as_slice(&self) -> &[T] {
        // SAFETY: the first `len` slots are initialized by every constructor
        // and `push`, and `len` can never exceed CAP.
        unsafe { core::slice::from_raw_parts(self.values.as_ptr().cast::<T>(), self.len) }
    }
}

impl<T, const MIN: usize, const CAP: usize> Drop for EvmCdArray<T, MIN, CAP> {
    fn drop(&mut self) {
        for value in &mut self.values[..self.len] {
            // SAFETY: exactly the first `len` slots are initialized.
            unsafe { value.assume_init_drop() };
        }
    }
}

impl<T: Clone, const MIN: usize, const CAP: usize> Clone for EvmCdArray<T, MIN, CAP> {
    fn clone(&self) -> Self {
        Self::try_from_slice(self.as_slice()).expect("an existing EvmCdArray has valid bounds")
    }
}

impl<T: core::fmt::Debug, const MIN: usize, const CAP: usize> core::fmt::Debug
    for EvmCdArray<T, MIN, CAP>
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.debug_list().entries(self.as_slice()).finish()
    }
}

impl<T: PartialEq, const MIN: usize, const CAP: usize> PartialEq for EvmCdArray<T, MIN, CAP> {
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl<T: Eq, const MIN: usize, const CAP: usize> Eq for EvmCdArray<T, MIN, CAP> {}

impl<T, const MIN: usize, const CAP: usize> AsRef<[T]> for EvmCdArray<T, MIN, CAP> {
    fn as_ref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<T, const MIN: usize, const CAP: usize> EvmCdSerialise for EvmCdArray<T, MIN, CAP>
where
    T: EvmCdSerialise,
{
    fn serialise<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        U::from_u32(32).serialise_value(writer)?;
        self.serialise_abi_tail(writer)
    }

    fn is_abi_dynamic() -> bool {
        true
    }

    fn abi_tail_size(&self) -> usize {
        self.len
            .saturating_mul(T::abi_head_size())
            .saturating_add(32)
            .saturating_add(self.as_slice().iter().fold(0usize, |size, value| {
                size.saturating_add(value.abi_tail_size())
            }))
    }

    fn serialise_abi_head<W: Write>(
        &self,
        tail_offset: usize,
        writer: &mut W,
    ) -> Result<(), Error> {
        U::from_usize(tail_offset).serialise_value(writer)
    }

    fn serialise_abi_tail<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        U::from_usize(self.len).serialise_value(writer)?;
        let mut tail_offset = self
            .len
            .checked_mul(T::abi_head_size())
            .ok_or_else(invalid_data)?;
        for value in self.as_slice() {
            value.serialise_abi_head(tail_offset, writer)?;
            tail_offset = tail_offset
                .checked_add(value.abi_tail_size())
                .ok_or_else(invalid_data)?;
        }
        for value in self.as_slice() {
            value.serialise_abi_tail(writer)?;
        }
        Ok(())
    }

    fn append_abi_type(hasher: SelectorHasher) -> SelectorHasher {
        T::append_abi_type(hasher).update(b"[]")
    }
}

impl<T, const MIN: usize, const CAP: usize> EvmCdDeserialise for EvmCdArray<T, MIN, CAP>
where
    T: EvmCdDeserialise,
{
    fn deserialise_reader<R: Read>(reader: &mut R) -> Result<Self, Error> {
        if read_usize_word(reader)? != 32 {
            return Err(invalid_data());
        }
        Self::deserialise_tail(reader)
    }

    fn is_abi_dynamic() -> bool {
        true
    }

    fn abi_tail_size(&self) -> usize {
        self.len
            .saturating_mul(T::abi_head_size())
            .saturating_add(32)
            .saturating_add(self.as_slice().iter().fold(0usize, |size, value| {
                size.saturating_add(value.abi_tail_size())
            }))
    }

    fn deserialise_abi_head<R: Read>(reader: &mut R) -> Result<EvmCdHead<Self>, Error> {
        Ok(EvmCdHead::Offset(read_usize_word(reader)?))
    }

    fn deserialise_abi_finish<R: Read>(
        head: EvmCdHead<Self>,
        expected_tail_offset: usize,
        reader: &mut R,
    ) -> Result<Self, Error> {
        match head {
            EvmCdHead::Offset(offset) if offset == expected_tail_offset => {
                Self::deserialise_tail(reader)
            }
            _ => Err(invalid_data()),
        }
    }

    fn append_abi_type(hasher: SelectorHasher) -> SelectorHasher {
        T::append_abi_type(hasher).update(b"[]")
    }
}

impl<T, const MIN: usize, const CAP: usize> EvmCdArray<T, MIN, CAP>
where
    T: EvmCdDeserialise,
{
    fn deserialise_tail<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let len = read_usize_word(reader)?;
        Self::validate_len(len).map_err(|_| invalid_data())?;
        let mut out = Self::empty();

        if T::is_abi_dynamic() {
            let mut offsets = [0usize; CAP];
            for offset in &mut offsets[..len] {
                match T::deserialise_abi_head(reader)? {
                    EvmCdHead::Offset(value) => *offset = value,
                    EvmCdHead::Value(_) => return Err(invalid_data()),
                }
            }
            let mut expected_tail_offset = len
                .checked_mul(T::abi_head_size())
                .ok_or_else(invalid_data)?;
            for offset in offsets[..len].iter().copied() {
                let value = T::deserialise_abi_finish(
                    EvmCdHead::Offset(offset),
                    expected_tail_offset,
                    reader,
                )?;
                expected_tail_offset = expected_tail_offset
                    .checked_add(value.abi_tail_size())
                    .ok_or_else(invalid_data)?;
                out.push(value);
            }
        } else {
            for _ in 0..len {
                let head = T::deserialise_abi_head(reader)?;
                out.push(T::deserialise_abi_finish(head, 0, reader)?);
            }
        }
        Ok(out)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvmCdStringError {
    InvalidBounds,
    TooShort,
    TooLong,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EvmCdString<const MIN: usize, const CAP: usize> {
    len: usize,
    bytes: [u8; CAP],
}

impl<const MIN: usize, const CAP: usize> EvmCdString<MIN, CAP> {
    pub fn try_from_str(value: &str) -> Result<Self, EvmCdStringError> {
        if MIN > CAP {
            return Err(EvmCdStringError::InvalidBounds);
        }
        if value.len() < MIN {
            return Err(EvmCdStringError::TooShort);
        }
        if value.len() > CAP {
            return Err(EvmCdStringError::TooLong);
        }
        let mut bytes = [0u8; CAP];
        bytes[..value.len()].copy_from_slice(value.as_bytes());
        Ok(Self {
            len: value.len(),
            bytes,
        })
    }

    pub const fn len(&self) -> usize {
        self.len
    }

    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub const fn capacity(&self) -> usize {
        CAP
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes[..self.len]
    }

    pub fn as_str(&self) -> &str {
        // SAFETY: constructors and deserialisation validate UTF-8, and no API
        // exposes mutable access to the initialized bytes.
        unsafe { core::str::from_utf8_unchecked(self.as_bytes()) }
    }
}

impl<const MIN: usize, const CAP: usize> TryFrom<&str> for EvmCdString<MIN, CAP> {
    type Error = EvmCdStringError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_from_str(value)
    }
}

impl<const MIN: usize, const CAP: usize> AsRef<str> for EvmCdString<MIN, CAP> {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<const MIN: usize, const CAP: usize> AsRef<[u8]> for EvmCdString<MIN, CAP> {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl<const MIN: usize, const CAP: usize> core::borrow::Borrow<str> for EvmCdString<MIN, CAP> {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl<const MIN: usize, const CAP: usize> core::fmt::Display for EvmCdString<MIN, CAP> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl<const MIN: usize, const CAP: usize> core::fmt::Debug for EvmCdString<MIN, CAP> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(self.as_str(), formatter)
    }
}

impl<const MIN: usize, const CAP: usize> core::str::FromStr for EvmCdString<MIN, CAP> {
    type Err = EvmCdStringError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::try_from_str(value)
    }
}

impl<const MIN: usize, const CAP: usize> EvmCdSerialise for EvmCdString<MIN, CAP> {
    fn serialise<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        write_dynamic_bytes(self.as_bytes(), writer)
    }

    fn is_abi_dynamic() -> bool {
        true
    }

    fn abi_tail_size(&self) -> usize {
        dynamic_tail_size(self.len)
    }

    fn serialise_abi_head<W: Write>(
        &self,
        tail_offset: usize,
        writer: &mut W,
    ) -> Result<(), Error> {
        U::from_usize(tail_offset).serialise_value(writer)
    }

    fn serialise_abi_tail<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        write_dynamic_tail(self.as_bytes(), writer)
    }

    fn append_abi_type(hasher: SelectorHasher) -> SelectorHasher {
        hasher.update(b"string")
    }
}

impl<const MIN: usize, const CAP: usize> EvmCdDeserialise for EvmCdString<MIN, CAP> {
    fn deserialise_reader<R: Read>(reader: &mut R) -> Result<Self, Error> {
        if MIN > CAP {
            return Err(invalid_data());
        }
        let (bytes, len) = read_dynamic_bytes::<CAP, _>(reader)?;
        if len < MIN || core::str::from_utf8(&bytes[..len]).is_err() {
            return Err(invalid_data());
        }
        Ok(Self { len, bytes })
    }

    fn is_abi_dynamic() -> bool {
        true
    }

    fn abi_tail_size(&self) -> usize {
        dynamic_tail_size(self.len)
    }

    fn deserialise_abi_head<R: Read>(reader: &mut R) -> Result<EvmCdHead<Self>, Error> {
        Ok(EvmCdHead::Offset(read_usize_word(reader)?))
    }

    fn deserialise_abi_finish<R: Read>(
        head: EvmCdHead<Self>,
        expected_tail_offset: usize,
        reader: &mut R,
    ) -> Result<Self, Error> {
        match head {
            EvmCdHead::Offset(offset) if offset == expected_tail_offset => {
                if MIN > CAP {
                    return Err(invalid_data());
                }
                let (bytes, len) = read_dynamic_tail::<CAP, _>(reader)?;
                if len < MIN || core::str::from_utf8(&bytes[..len]).is_err() {
                    return Err(invalid_data());
                }
                Ok(Self { len, bytes })
            }
            _ => Err(invalid_data()),
        }
    }

    fn append_abi_type(hasher: SelectorHasher) -> SelectorHasher {
        hasher.update(b"string")
    }
}

#[cfg(feature = "alloc")]
impl<const MIN: usize, const CAP: usize> From<EvmCdString<MIN, CAP>> for String {
    fn from(value: EvmCdString<MIN, CAP>) -> Self {
        String::from(value.as_str())
    }
}

#[cfg(feature = "alloc")]
impl EvmCdSerialise for Vec<u8> {
    fn serialise<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        write_dynamic_bytes(self, writer)
    }

    fn is_abi_dynamic() -> bool {
        true
    }

    fn abi_tail_size(&self) -> usize {
        dynamic_tail_size(self.len())
    }

    fn serialise_abi_head<W: Write>(
        &self,
        tail_offset: usize,
        writer: &mut W,
    ) -> Result<(), Error> {
        U::from_usize(tail_offset).serialise_value(writer)
    }

    fn serialise_abi_tail<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        write_dynamic_tail(self, writer)
    }

    fn append_abi_type(hasher: SelectorHasher) -> SelectorHasher {
        hasher.update(b"bytes")
    }
}

#[cfg(feature = "alloc")]
fn read_vec_tail<R: Read>(reader: &mut R) -> Result<Vec<u8>, Error> {
    const MAX_ALLOC_DESERIALISE_LEN: usize = 16 * 1024 * 1024;
    let len = read_usize_word(reader)?;
    if len > MAX_ALLOC_DESERIALISE_LEN {
        return Err(invalid_data());
    }
    let mut out = Vec::new();
    out.try_reserve_exact(len).map_err(|_| invalid_data())?;
    out.resize(len, 0);
    reader.read_exact(&mut out)?;
    let padding = (32 - len % 32) % 32;
    let mut padding_bytes = [0u8; 31];
    reader.read_exact(&mut padding_bytes[..padding])?;
    if padding_bytes[..padding].iter().any(|byte| *byte != 0) {
        return Err(invalid_data());
    }
    Ok(out)
}

#[cfg(feature = "alloc")]
impl EvmCdDeserialise for Vec<u8> {
    fn deserialise_reader<R: Read>(reader: &mut R) -> Result<Self, Error> {
        if read_usize_word(reader)? != 32 {
            return Err(invalid_data());
        }
        read_vec_tail(reader)
    }

    fn is_abi_dynamic() -> bool {
        true
    }

    fn abi_tail_size(&self) -> usize {
        dynamic_tail_size(self.len())
    }

    fn deserialise_abi_head<R: Read>(reader: &mut R) -> Result<EvmCdHead<Self>, Error> {
        Ok(EvmCdHead::Offset(read_usize_word(reader)?))
    }

    fn deserialise_abi_finish<R: Read>(
        head: EvmCdHead<Self>,
        expected_tail_offset: usize,
        reader: &mut R,
    ) -> Result<Self, Error> {
        match head {
            EvmCdHead::Offset(offset) if offset == expected_tail_offset => read_vec_tail(reader),
            _ => Err(invalid_data()),
        }
    }

    fn append_abi_type(hasher: SelectorHasher) -> SelectorHasher {
        hasher.update(b"bytes")
    }
}
