use crate::reader::{ReadError, Readable, Reader, Result};
use crate::types::*;
use crate::writer::Writable;
use num_traits::FromPrimitive;

fn pad_write(out: &mut Vec<u8>) {
    let pad_bytes = [0; 4];
    let p = (4 - (out.len() % 4)) % 4;
    out.extend_from_slice(&pad_bytes[..p]);
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

macro_rules! impl_number {
    ($ty:ty) => {
        impl<'a> Readable<'a> for $ty {
            fn read(reader: &mut Reader<'a>) -> Result<Self> {
                read_int!(reader, $ty)
            }
        }

        impl Writable for $ty {
            fn write(&self, out: &mut Vec<u8>) {
                out.extend_from_slice(&self.to_ne_bytes());
            }
        }
    };
}

impl<'a> Readable<'a> for u8 {
    fn read(reader: &mut Reader<'a>) -> Result<Self> {
        reader.u8()
    }
}

impl Writable for u8 {
    fn write(&self, out: &mut Vec<u8>) {
        out.push(*self);
    }
}

impl_number!(u16);
impl_number!(u32);
impl_number!(i32);

macro_rules! impl_enum {
    ($ty:ty, $repr:ty) => {
        impl<'a> Readable<'a> for $ty {
            fn read(reader: &mut Reader<'a>) -> Result<Self> {
                let repr = <$repr>::read(reader)?;
                <$ty as FromPrimitive>::from_u32(repr as u32).ok_or(ReadError::InvalidData)
            }
        }

        impl Writable for $ty {
            fn write(&self, out: &mut Vec<u8>) {
                unsafe { (*(self as *const Self as *const $repr)).write(out) }
            }
        }
    };
}

impl_enum!(Opcode, u8);
impl_enum!(CaretDirection, u32);
impl_enum!(CaretStyle, u32);
impl_enum!(StrConvFeedbackType, u16);
impl_enum!(Feedback, u32);
impl_enum!(PreeditState, u32);
impl_enum!(HotkeyState, u32);
impl_enum!(ResetState, u32);

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

        impl Writable for $ty {
            fn write(&self, out: &mut Vec<u8>) {
                $(
                    self.$field.write(out);
                )+
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

        impl<'a> Writable for $ty<'a> {
            fn write(&self, out: &mut Vec<u8>) {
                $(
                    self.$field.write(out);
                )+
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
impl_struct!(RequestHeader, major_opcode, minor_opcode, length);
impl_struct!(@Attr, id, type_, name);
impl_struct!(
    ConnectReply,
    server_major_protocol_version,
    server_minor_protocol_version
);

impl<'a> Readable<'a> for XimString<'a> {
    fn read(reader: &mut Reader<'a>) -> Result<Self> {
        let len = reader.u16()?;
        let string = reader.string(len as usize)?;
        reader.pad();
        Ok(XimString(string))
    }
}

impl<'a> Writable for XimString<'a> {
    fn write(&self, out: &mut Vec<u8>) {
        (self.0.len() as u16).write(out);
        out.extend_from_slice(self.0.as_bytes());
        pad_write(out);
    }
}

impl<'a> Readable<'a> for Connect<'a> {
    fn read(reader: &mut Reader<'a>) -> Result<Self> {
        let endian = reader.u8()?;

        match (
            endian,
            cfg!(target_endian = "big"),
            cfg!(target_endian = "little"),
        ) {
            (b'\x6c', _, true) | (b'\x42', true, _) => {}
            (_, _, _) => return Err(ReadError::InvalidData),
        }

        let major_ver = reader.u16()?;
        let minor_ver = reader.u16()?;
        let protocol_count = reader.u16()?;

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
        let header = RequestHeader::read(reader)?;

        let this = match header.major_opcode {
            Opcode::Connect => Request::Connect(Connect::read(reader)?),
            Opcode::ConnectReply => Request::ConnectReply(ConnectReply::read(reader)?),
            _ => todo!(),
        };

        Ok(this)
    }
}

impl<'a> Writable for Request<'a> {
    fn write(&self, out: &mut Vec<u8>) {
        let header = RequestHeader {
            major_opcode: self.opcode(),
            minor_opcode: 0,
            length: 0,
        };

        header.write(out);

        let prev = out.len();

        match self {
            // Request::Connect(connect) => connect.write(out),
            Request::ConnectReply(reply) => reply.write(out),
            _ => todo!(),
        }

        let size = out.len() - prev;

        let len_byte = (size as u16).to_ne_bytes();

        out[(prev - 2)..prev].copy_from_slice(&len_byte);
    }
}
