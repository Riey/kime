#![allow(unused_variables)]

use crate::{Endianness, C16, C8, C32};
use serde::de::{DeserializeSeed, SeqAccess, Visitor};
use serde::*;
use std::io;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ReadError {
    #[error("End of Stream")]
    EndOfStream,
    #[error("Utf8 error: {0}")]
    Utf8(#[from] std::str::Utf8Error),
    #[error("Serde error: {0}")]
    Serde(String),
}

pub type Result<T> = std::result::Result<T, ReadError>;

impl de::Error for ReadError {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        ReadError::Serde(msg.to_string())
    }
}

pub struct Reader<'a> {
    b: &'a [u8],
    start: usize,
    endian: Endianness,
}

macro_rules! read_int {
    ($self:expr, $ty:ty) => {
        if $self.b.len() < std::mem::size_of::<$ty>() {
            Err(ReadError::EndOfStream)
        } else {
            use std::convert::TryInto;
            let (bytes, b) = $self.b.split_at(std::mem::size_of::<$ty>());
            $self.b = b;
            let arr: [u8; std::mem::size_of::<$ty>()] = bytes
                .try_into()
                .unwrap_or_else(|_| unsafe { std::hint::unreachable_unchecked() });
            dbg!(arr);
            Ok(match $self.endian {
                Endianness::Little => <$ty>::from_le_bytes(arr),
                Endianness::Big => <$ty>::from_be_bytes(arr),
                Endianness::Native => <$ty>::from_ne_bytes(arr),
            })
        }
    };
}

impl<'a> Reader<'a> {
    pub fn new(b: &'a [u8], endian: Endianness) -> Self {
        Self { b, start: b.as_ptr() as usize, endian }
    }

    fn c8(&mut self) -> Result<C8> {
        match self.b {
            [b, other @ ..] => {
                self.b = other;
                Ok(*b)
            }
            [] => Err(ReadError::EndOfStream),
        }
    }

    fn c16(&mut self) -> Result<C16> {
        read_int!(self, C16)
    }

    fn c32(&mut self) -> Result<C32> {
        read_int!(self, C32)
    }

    fn ptr_offset(&self) -> usize {
        self.b.as_ptr() as usize - self.start
    }

    fn pad(&mut self) {
        let p = (4 - (self.ptr_offset() % 4)) % 4;
        self.b = &self.b[p..];
    }

    fn string(&mut self) -> Result<&'a str> {
        let len = self.c16()? as usize;
        let (bytes, left) = self.b.split_at(len);
        self.pad();
        self.b = left;
        Ok(std::str::from_utf8(bytes)?)
    }

    fn feedback(&mut self) -> Result<Vec<u32>> {
        let m = self.c16()? as usize;
        let _ = self.c16()?;

        let mut ret = Vec::with_capacity(m);

        for _ in 0..m {
            ret.push(self.c32()?);
        }

        Ok(ret)
    }
}

impl<'a, 'de> SeqAccess<'de> for &'a mut Reader<'de> {
    type Error = ReadError;

    fn next_element_seed<T>(
        &mut self,
        seed: T,
    ) -> Result<Option<<T as DeserializeSeed<'de>>::Value>>
    where
        T: DeserializeSeed<'de>,
    {
        if self.b.is_empty() {
            Ok(None)
        } else {
            seed.deserialize(&mut **self).map(Some)
        }
    }
}

impl<'a, 'de> Deserializer<'de> for &'a mut Reader<'de> {
    type Error = ReadError;

    fn deserialize_any<V>(self, visitor: V) -> Result<<V as Visitor<'de>>::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<<V as Visitor<'de>>::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<<V as Visitor<'de>>::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i8(self.c8()? as i8)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<<V as Visitor<'de>>::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i16(read_int!(self, i16)?)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<<V as Visitor<'de>>::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i32(read_int!(self, i32)?)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<<V as Visitor<'de>>::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i64(read_int!(self, i64)?)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<<V as Visitor<'de>>::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u8(self.c8()?)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<<V as Visitor<'de>>::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u16(read_int!(self, u16)?)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<<V as Visitor<'de>>::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u32(read_int!(self, u32)?)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<<V as Visitor<'de>>::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u64(read_int!(self, u64)?)
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<<V as Visitor<'de>>::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<<V as Visitor<'de>>::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<<V as Visitor<'de>>::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<<V as Visitor<'de>>::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_borrowed_str(self.string()?)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<<V as Visitor<'de>>::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_string(self.string()?.to_string())
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<<V as Visitor<'de>>::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<<V as Visitor<'de>>::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<<V as Visitor<'de>>::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<<V as Visitor<'de>>::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_unit_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<<V as Visitor<'de>>::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_newtype_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<<V as Visitor<'de>>::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<<V as Visitor<'de>>::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_seq(self)
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<<V as Visitor<'de>>::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_tuple_struct<V>(
        self,
        name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<<V as Visitor<'de>>::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<<V as Visitor<'de>>::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_struct<V>(
        self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<<V as Visitor<'de>>::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    fn deserialize_enum<V>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<<V as Visitor<'de>>::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<<V as Visitor<'de>>::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<<V as Visitor<'de>>::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }
}
