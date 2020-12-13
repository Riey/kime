use crate::reader::{ReadError, Readable, Reader, Result};
use crate::types::*;
use crate::writer::{Writable, Writer};
use num_traits::FromPrimitive;
use std::marker::PhantomData;

macro_rules! read_int {
    ($self:expr, $ty:ty) => {{
        use std::convert::TryInto;
        let bytes = $self.consume(std::mem::size_of::<$ty>())?;
        let arr: [u8; std::mem::size_of::<$ty>()] = bytes
            .try_into()
            .unwrap_or_else(|_| unsafe { std::hint::unreachable_unchecked() });
        Ok(<$ty>::from_ne_bytes(arr))
    }};
}

macro_rules! impl_number {
    ($ty:ty) => {
        impl<'a> Readable<'a> for $ty {
            fn read(reader: &mut Reader<'a>) -> Result<Self> {
                read_int!(reader, $ty)
            }
        }

        impl Writable for $ty {
            fn write(&self, writer: &mut Writer) {
                writer.write(&self.to_ne_bytes());
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
    fn write(&self, writer: &mut Writer) {
        writer.u8(*self);
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
                <$ty as FromPrimitive>::from_u32(repr as u32)
                    .ok_or_else(|| reader.invalid_data(stringify!($ty), repr))
            }
        }

        impl Writable for $ty {
            fn write(&self, writer: &mut Writer) {
                (*self as $repr).write(writer)
            }
        }
    };
}

impl_enum!(CaretDirection, u32);
impl_enum!(CaretStyle, u32);
impl_enum!(StrConvFeedbackType, u16);
impl_enum!(Feedback, u32);
impl_enum!(AttrType, u16);
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
            fn write(&self, writer: &mut Writer) {
                $(
                    self.$field.write(writer);
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
            fn write(&self, writer: &mut Writer) {
                $(
                    self.$field.write(writer);
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
impl_struct!(RequestHeader, major_opcode, minor_opcode, size);
impl_struct!(
    @ConnectReply,
    server_major_protocol_version,
    server_minor_protocol_version,
    _marker
);

impl<'a, T> Readable<'a> for PhantomData<T> {
    #[inline(always)]
    fn read(_reader: &mut Reader<'a>) -> Result<Self> {
        Ok(PhantomData)
    }
}

impl<T> Writable for PhantomData<T> {
    #[inline(always)]
    fn write(&self, _writer: &mut Writer) {}
}

impl<'a> Readable<'a> for XimString<'a> {
    fn read(reader: &mut Reader<'a>) -> Result<Self> {
        let len = reader.u16()?;
        let string = reader.consume(len)?;
        Ok(XimString(string))
    }
}

impl<'a> Writable for XimString<'a> {
    fn write(&self, writer: &mut Writer) {
        (self.0.len() as u16).write(writer);
        writer.write(self.0);
    }
}

impl<'a> Readable<'a> for XimStr<'a> {
    fn read(reader: &mut Reader<'a>) -> Result<Self> {
        let len = reader.u8()?;
        let string = reader.consume(len)?;
        Ok(XimStr(string))
    }
}

impl<'a> Writable for XimStr<'a> {
    fn write(&self, writer: &mut Writer) {
        (self.0.len() as u8).write(writer);
        writer.write(self.0);
    }
}

impl<'a> Readable<'a> for Connect<'a> {
    fn read(reader: &mut Reader<'a>) -> Result<Self> {
        let endian = reader.u8()?;
        let _unused = reader.u8()?;

        match (
            endian,
            cfg!(target_endian = "big"),
            cfg!(target_endian = "little"),
        ) {
            (b'\x6c', _, true) | (b'\x42', true, _) => {}
            (_, _, _) => return Err(reader.invalid_data("Endian", endian)),
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

impl<'a> Writable for Connect<'a> {
    fn write(&self, _writer: &mut Writer) {
        unimplemented!()
    }
}

impl<'a> Readable<'a> for Attr<'a> {
    fn read(reader: &mut Reader<'a>) -> Result<Self> {
        let id = u16::read(reader)?;
        let type_ = AttrType::read(reader)?;
        let name = XimString::read(reader)?;
        reader.pad();

        Ok(Self { id, type_, name })
    }
}

impl<'a> Writable for Attr<'a> {
    fn write(&self, writer: &mut Writer) {
        self.id.write(writer);
        self.type_.write(writer);
        self.name.write(writer);
        writer.pad();
    }
}

impl<'a> Readable<'a> for OpenReply<'a> {
    fn read(reader: &mut Reader<'a>) -> Result<Self> {
        let id = reader.u16()?;
        let len = reader.u16()? as usize;
        let mut xim_attributes = Vec::new();

        let target = reader.bytes_len() - len;
        while reader.bytes_len() > target {
            xim_attributes.push(Attr::read(reader)?);
        }

        let len = reader.u16()? as usize;
        reader.u16()?;
        let mut xic_attributes = Vec::new();

        let target = reader.bytes_len() - len;
        while reader.bytes_len() > target {
            xic_attributes.push(Attr::read(reader)?);
        }

        Ok(Self {
            input_method_id: id,
            xim_attributes,
            xic_attributes,
        })
    }
}

impl<'a> Writable for OpenReply<'a> {
    fn write(&self, writer: &mut Writer) {
        self.input_method_id.write(writer);

        0u16.write(writer);
        let mark = writer.mark_len();
        for attr in self.xim_attributes.iter() {
            attr.write(writer);
        }

        writer.write_u16_len(mark);

        0u16.write(writer);
        let mark = writer.mark_len();

        0u16.write(writer);
        for attr in self.xic_attributes.iter() {
            attr.write(writer);
        }

        // sub 2 byte for ignore unused
        writer.write_u16_len_sub(mark, 2);
    }
}

impl<'a> Readable<'a> for QueryExtension<'a> {
    fn read(reader: &mut Reader<'a>) -> Result<Self> {
        let id = reader.u16()?;
        let len = reader.u16()? as usize;

        let mut extensions = Vec::new();

        let target = reader.bytes_len() - len;
        while reader.bytes_len() > target {
            extensions.push(XimStr::read(reader)?);
        }

        reader.pad();

        Ok(Self {
            input_method_id: id,
            extensions,
        })
    }
}

impl<'a> Writable for QueryExtension<'a> {
    fn write(&self, writer: &mut Writer) {
        self.input_method_id.write(writer);
    }
}

impl<'a> Readable<'a> for QueryExtensionReply<'a> {
    fn read(reader: &mut Reader<'a>) -> Result<Self> {
        let id = reader.u16()?;
        let len = reader.u16()? as usize;

        let mut extensions = Vec::new();

        let target = reader.bytes_len() - len;
        while reader.bytes_len() > target {
            extensions.push(Extension::read(reader)?);
        }

        reader.pad();

        Ok(Self {
            input_method_id: id,
            extensions,
        })
    }
}

impl<'a> Writable for QueryExtensionReply<'a> {
    fn write(&self, writer: &mut Writer) {
        self.input_method_id.write(writer);
        0u16.write(writer);
        let mark = writer.mark_len();
        for ex in self.extensions.iter() {
            ex.write(writer);
        }
        writer.write_u16_len(mark);
        writer.pad();
    }
}

impl<'a> Readable<'a> for Open<'a> {
    fn read(reader: &mut Reader<'a>) -> Result<Self> {
        let name = XimStr::read(reader)?;
        reader.pad();
        Ok(Self { name })
    }
}

impl<'a> Writable for Open<'a> {
    fn write(&self, writer: &mut Writer) {
        self.name.write(writer);
        writer.pad();
    }
}

impl<'a> Readable<'a> for Extension<'a> {
    fn read(reader: &mut Reader<'a>) -> Result<Self> {
        let major_opcode = reader.u8()?;
        let minor_opcode = reader.u8()?;
        let name = XimString::read(reader)?;
        reader.pad();

        Ok(Self {
            major_opcode,
            minor_opcode,
            name,
        })
    }
}

impl<'a> Writable for Extension<'a> {
    fn write(&self, out: &mut Writer) {
        self.major_opcode.write(out);
        self.minor_opcode.write(out);
        self.name.write(out);
        out.pad();
    }
}
