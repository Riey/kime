use crate::reader::{ReadError, Readable, Reader, Result};
use crate::types::*;
use crate::writer::Writable;
use num_traits::FromPrimitive;
use std::marker::PhantomData;

fn pad_size(len: impl Into<usize>) -> usize {
    (4 - (len.into() % 4)) % 4
}

fn pad_write(out: &mut Vec<u8>) {
    let pad_bytes = [0; 4];
    let p = pad_size(out.len());
    out.extend_from_slice(&pad_bytes[..p]);
}

macro_rules! read_int {
    ($self:expr, $ty:ty) => {
        if $self.b.len() < std::mem::size_of::<$ty>() {
            Err($self.eos())
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
            fn size(&self) -> usize {
                std::mem::size_of::<$ty>()
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

    fn size(&self) -> usize {
        1
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
                <$ty as FromPrimitive>::from_u32(repr as u32).ok_or_else(|| reader.invalid_data(stringify!($ty), repr))
            }
        }

        impl Writable for $ty {
            fn write(&self, out: &mut Vec<u8>) {
                (*self as $repr).write(out)
            }

            fn size(&self) -> usize {
                (*self as $repr).size()
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
            fn write(&self, out: &mut Vec<u8>) {
                $(
                    self.$field.write(out);
                )+
            }
            fn size(&self) -> usize {
                let mut len = 0;
                $(
                    len += self.$field.size();
                )+
                len
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
            fn size(&self) -> usize {
                let mut len = 0;
                $(
                    len += self.$field.size();
                )+
                len
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
impl_struct!(@Open, name);
impl_struct!(@Extension, major_opcode, minor_opcode, name);

impl<'a, T> Readable<'a> for PhantomData<T> {
    #[inline(always)]
    fn read(_reader: &mut Reader<'a>) -> Result<Self> {
        Ok(PhantomData)
    }
}

impl<T> Writable for PhantomData<T> {
    #[inline(always)]
    fn write(&self, _out: &mut Vec<u8>) {}

    #[inline(always)]
    fn size(&self) -> usize {
        0
    }
}

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
        out.extend_from_slice(self.0);
        pad_write(out);
    }
    fn size(&self) -> usize {
        let size = 2 + self.0.len();
        size + pad_size(size)
    }
}

impl<'a> Readable<'a> for XimStr<'a> {
    fn read(reader: &mut Reader<'a>) -> Result<Self> {
        let len = reader.u8()?;
        let string = reader.string(len as usize)?;
        Ok(XimStr(string))
    }
}

impl<'a> Writable for XimStr<'a> {
    fn write(&self, out: &mut Vec<u8>) {
        (self.0.len() as u8).write(out);
        out.extend_from_slice(self.0);
    }
    fn size(&self) -> usize {
        self.0.len() + 1
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
    fn write(&self, _out: &mut Vec<u8>) {
        unimplemented!()
    }

    fn size(&self) -> usize {
        self.client_auth_protocol_names
            .iter()
            .map(Writable::size)
            .sum::<usize>()
            + 8
    }
}

impl<'a> Readable<'a> for Attr<'a> {
    fn read(reader: &mut Reader<'a>) -> Result<Self> {
        let id = u16::read(reader)?;
        let type_ = AttrType::read(reader)?;
        let name = XimString::read(reader)?;

        Ok(Self {
            id,
            type_,
            name,
        })
    }
}

impl<'a> Writable for Attr<'a> {
    fn write(&self, out: &mut Vec<u8>) {}
    fn size(&self) -> usize { 0 }
}

impl<'a> Readable<'a> for OpenReply<'a> {
    fn read(reader: &mut Reader<'a>) -> Result<Self> {
        let id = reader.u16()?;
        let n = reader.u16()?;
        let mut xim = reader.cut(n as usize);
        let mut xim_attributes = Vec::new();

        while !xim.b.is_empty() {
            xim_attributes.push(Readable::read(&mut xim)?);
        }

        let m = reader.u16()?;
        let mut xic = reader.cut(m as usize);
        let mut xic_attributes = Vec::new();

        while !xic.b.is_empty() {
            xic_attributes.push(Readable::read(&mut xic)?);
        }

        Ok(Self {
            input_method_id: id,
            xim_attributes,
            xic_attributes,
        })
    }
}

impl<'a> Writable for OpenReply<'a> {
    fn write(&self, out: &mut Vec<u8>) {
        self.input_method_id.write(out);

        let n = self
            .xim_attributes
            .iter()
            .map(Writable::size)
            .sum::<usize>() as u16;
        let m = self
            .xic_attributes
            .iter()
            .map(Writable::size)
            .sum::<usize>() as u16;

        n.write(out);
        for attr in self.xim_attributes.iter() {
            attr.write(out);
        }

        m.write(out);
        0u16.write(out);
        for attr in self.xic_attributes.iter() {
            attr.write(out);
        }
    }

    fn size(&self) -> usize {
        self.xim_attributes
            .iter()
            .map(Writable::size)
            .sum::<usize>()
            + self
                .xic_attributes
                .iter()
                .map(Writable::size)
                .sum::<usize>()
            + 8
    }
}

impl<'a> Readable<'a> for QueryExtension<'a> {
    fn read(reader: &mut Reader<'a>) -> Result<Self> {
        let id = reader.u16()?;
        let len = reader.u16()?;

        let mut ex = reader.cut(len as usize);

        let mut extensions = Vec::new();

        while !ex.is_empty() {
            extensions.push(XimStr::read(&mut ex)?);
        }

        reader.pad();

        Ok(Self {
            input_method_id: id,
            extensions,
        })
    }
}

impl<'a> Writable for QueryExtension<'a> {
    fn write(&self, out: &mut Vec<u8>) {
        self.input_method_id.write(out);
    }

    fn size(&self) -> usize {
        let len = 4 + self.extensions.iter().map(Writable::size).sum::<usize>();
        len + pad_size(len)
    }
}

impl<'a> Readable<'a> for QueryExtensionReply<'a> {
    fn read(reader: &mut Reader<'a>) -> Result<Self> {
        let id = reader.u16()?;
        let len = reader.u16()?;
        reader.b = &reader.b[..len as usize];

        let mut extensions = Vec::new();

        while !reader.b.is_empty() {
            extensions.push(Extension::read(reader)?);
        }

        Ok(Self {
            input_method_id: id,
            extensions,
        })
    }
}

impl<'a> Writable for QueryExtensionReply<'a> {
    fn write(&self, out: &mut Vec<u8>) {
        self.input_method_id.write(out);
        let size = self.extensions.iter().map(Writable::size).sum::<usize>() as u16;
        size.write(out);
        for ex in self.extensions.iter() {
            ex.write(out);
        }
    }

    fn size(&self) -> usize {
        4 + self.extensions.iter().map(Writable::size).sum::<usize>()
    }
}
