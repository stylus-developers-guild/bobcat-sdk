// These functions are based heavily on the approach borsh took with it's
// io crate. Though in our version, we miss the interruption code.

use bobcat_maths::U;

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

#[cfg(not(feature = "std"))]
mod no_std {
    #[derive(Debug, Clone, PartialEq)]
    pub enum Error {
        WriteAllEof,
        ReadExactEof,
    }

    pub trait Write {
        fn write(&mut self, buf: &[u8]) -> Result<usize, Error>;
        fn flush(&mut self) -> Result<(), Error>;
        fn is_empty(&self) -> bool;

        fn write_all(&mut self, mut buf: &[u8]) -> Result<(), Error> {
            while !buf.is_empty() {
                match self.write(buf) {
                    Ok(0) => {
                        return Err(Error::WriteAllEof);
                    }
                    Ok(n) => buf = &buf[n..],
                    Err(e) => return Err(e),
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
                    Ok(n) => {
                        buf = &mut buf[n..];
                    }
                    Err(e) => return Err(e),
                }
            }
            if !buf.is_empty() {
                Err(Error::ReadExactEof)
            } else {
                Ok(())
            }
        }
    }
}

#[cfg(not(feature = "std"))]
pub use no_std::{Error, Read, Write};

#[cfg(feature = "std")]
pub use std::io::{Error, Read, Write};

pub trait EvmCdSerialise {
    fn serialise<W: Write>(&self, writer: &mut W) -> Result<(), Error>;
}

pub trait EvmCdDeserialise: Sized {
    fn deserialise_reader<R: Read>(reader: &mut R) -> Result<Self, Error>;
}

impl EvmCdSerialise for U {
    fn serialise<W: Write>(&self, w: &mut W) -> Result<(), Error> {
        w.write_all(&self.0)
    }
}

impl EvmCdDeserialise for U {
    fn deserialise_reader<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let mut buf = [0u8; 32];
        reader.read_exact(&mut buf)?;
        Ok(U(buf))
    }
}

macro_rules! for_ints {
    ($($t:ty),+ $(,)?) => {
        $(
            impl EvmCdSerialise for $t {
                fn serialise<W: Write>(&self, w: &mut W) -> Result<(), Error> {
                    w.write_all(&[0u8; 32-size_of::<$t>()])?;
                    w.write_all(&self.to_be_bytes())
                }
            }

            impl EvmCdDeserialise for $t {
                fn deserialise_reader<R: Read>(reader: &mut R) -> Result<Self, Error> {
                    let U(u) = U::deserialise_reader(reader)?;
                    Ok(<$t>::from_be_bytes(u[32-size_of::<$t>()..].try_into().unwrap()))
                }
            }
        )+
    };
}

for_ints! { u8, u16, u32, u64, u128, usize }

impl<const N: usize> EvmCdSerialise for [u8; N] {
    fn serialise<W: Write>(&self, w: &mut W) -> Result<(), Error> {
        w.write_all(self)
    }
}

impl<const N: usize> EvmCdDeserialise for [u8; N] {
    fn deserialise_reader<R: Read>(r: &mut R) -> Result<Self, Error> {
        let mut out = [0u8; N];
        r.read_exact(&mut out)?;
        Ok(out)
    }
}

#[cfg(feature = "alloc")]
impl EvmCdSerialise for Vec<u8> {
    fn serialise<W: Write>(&self, w: &mut W) -> Result<(), Error> {
        w.write_all(&U::from_u32(32).0)?;
        w.write_all(&U::from_usize(N).0)?;
        w.write_all(self)?;
        Ok(())
    }
}

#[cfg(feature = "alloc")]
impl EvmCdDeserialise for Vec<u8> {
    fn deserialise_reader<R: Read>(r: &mut R) -> Result<Self, Error> {
        let mut head = [0u8; 64];
        r.read_exact(&mut head)?;
        let mut out =
            Vec::with_capacity(usize::from_be_bytes(head[32..32 * 2].try_into().unwrap()));
        r.read_exact(&mut out)?;
        Ok(out)
    }
}
