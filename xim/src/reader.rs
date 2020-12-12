#![allow(unused_variables)]

use crate::types::*;
use num_traits::FromPrimitive;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ReadError {
    #[error("End of Stream")]
    EndOfStream,
    #[error("Utf8 error: {0}")]
    Utf8(#[from] std::str::Utf8Error),
    #[error("Invalid Data")]
    InvalidData,
}

pub type Result<T> = std::result::Result<T, ReadError>;

pub struct Reader<'a> {
    b: &'a [u8],
    start: usize,
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
            Ok(<$ty>::from_ne_bytes(arr))
        }
    };
}

impl<'a> Reader<'a> {
    pub fn new(b: &'a [u8]) -> Self {
        Self {
            b,
            start: b.as_ptr() as usize,
        }
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

    fn string(&mut self, len: usize) -> Result<&'a str> {
        let bytes = self.bytes(len)?;
        Ok(std::str::from_utf8(bytes)?)
    }

    fn bytes(&mut self, len: usize) -> Result<&'a [u8]> {
        if self.b.len() < len {
            Err(ReadError::EndOfStream)
        } else {
            let (bytes, left) = self.b.split_at(len);
            self.b = left;
            Ok(bytes)
        }
    }
}

pub trait Readable<'a>: Sized {
    fn read(reader: &mut Reader<'a>) -> Result<Self>;
}

macro_rules! impl_number {
    (C8) => {
        impl<'a> Readable<'a> for C8 {
            fn read(reader: &mut Reader<'a>) -> Result<Self> {
                reader.c8()
            }
        }
    };
    ($ty:ty) => {
        impl<'a> Readable<'a> for $ty {
            fn read(reader: &mut Reader<'a>) -> Result<Self> {
                read_int!(reader, $ty)
            }
        }
    };
}

impl_number!(C8);
impl_number!(C16);
impl_number!(C32);
impl_number!(i32);

macro_rules! impl_enum {
    ($ty:ty, $repr:ty) => {
        impl<'a> Readable<'a> for $ty {
            fn read(reader: &mut Reader<'a>) -> Result<Self> {
                let repr = <$repr>::read(reader)?;
                <$ty as FromPrimitive>::from_u32(repr as u32).ok_or(ReadError::InvalidData)
            }
        }
    };
}

impl_enum!(Opcode, C8);
impl_enum!(CaretDirection, C32);
impl_enum!(CaretStyle, C32);
impl_enum!(StrConvFeedbackType, C16);
impl_enum!(Feedback, C32);
impl_enum!(PreeditState, C32);
impl_enum!(HotkeyState, C32);
impl_enum!(ResetState, C32);

macro_rules! impl_struct {
    ($ty:ident, $($field:ident),+) => {
        impl<'a> Readable<'a> for $ty {
            fn read(reader: &mut Reader<'a>) -> Result<Self> {
                Ok($ty {
                    $(
                        $field: Readable::read(reader)?,
                    )+
                })
            }
        }
    };
    (@$ty:ident, $($field:ident),+) => {
        impl<'a> Readable<'a> for $ty<'a> {
            fn read(reader: &mut Reader<'a>) -> Result<Self> {
                Ok($ty {
                    $(
                        $field: Readable::read(reader)?,
                    )+
                })
            }
        }
    };
}

impl_struct!(
    PreeditCaret,
    method_id,
    context_id,
    position,
    direction,
    style
);
impl_struct!(PreeditCaretReply, method_id, context_id, position);
impl_struct!(PreeditDone, method_id, context_id);
impl_struct!(RequestPacketHeader, major_opcode, minor_opcode, length);
impl_struct!(@Attr, id, type_, name);

impl<'a> Readable<'a> for XimString<'a> {
    fn read(reader: &mut Reader<'a>) -> Result<Self> {
        let len = reader.c16()?;
        let string = reader.string(len as usize)?;
        reader.pad();
        Ok(XimString(string))
    }
}

impl<'a> Readable<'a> for Connect<'a> {
    fn read(reader: &mut Reader<'a>) -> Result<Self> {
        let endian = reader.c8()?;

        match (
            endian,
            cfg!(target_endian = "big"),
            cfg!(target_endian = "little"),
        ) {
            (b'\x6c', _, true) | (b'\x42', true, _) => {}
            (_, _, _) => return Err(ReadError::InvalidData),
        }

        let major_ver = reader.c16()?;
        let minor_ver = reader.c16()?;
        let protocol_count = reader.c16()?;

        let mut names = Vec::with_capacity(protocol_count as usize);

        for _ in 0..protocol_count {
            names.push(XimString::read(reader)?);
        }

        Ok(Self {
            client_major_protocol_version: major_ver,
            client_minor_protocol_version: minor_ver,
            client_auth_protocol_names: names,
        })
    }
}

impl<'a> Readable<'a> for Request<'a> {
    fn read(reader: &mut Reader<'a>) -> Result<Self> {
        let header = RequestPacketHeader::read(reader)?;

        match header.major_opcode {
            Opcode::Connect => Ok(Request::Connect(Connect::read(reader)?)),
            _ => todo!(),
        }
    }
}
