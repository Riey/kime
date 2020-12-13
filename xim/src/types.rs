use crate::reader::{ReadError, Readable, Reader, Result};
use crate::writer::Writable;
use enumflags2::BitFlags;
use num_derive::FromPrimitive;
use std::marker::PhantomData;

pub type WINDOW = u32;

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub struct RequestHeader {
    pub major_opcode: Opcode,
    pub minor_opcode: u8,
    pub size: u16,
}

macro_rules! define_request {
    (
        $(
            $op:ident = $n:literal,
        )+
    ) => {
        #[derive(Eq, PartialEq, Copy, Clone, Debug)]
        pub enum Opcode {
            $($op = $n,)+
        }

        #[derive(Eq, PartialEq, Clone, Debug)]
        pub enum Request<'a> {
            $($op($op<'a>),)+
        }

        impl<'a> Request<'a> {
            fn content_size(&self) -> usize {
                match self {
                    $(Request::$op(req) => req.size(),)+
                }
            }

            fn opcode(&self) -> Opcode {
                match self {
                    $(Request::$op(req) => Opcode::$op,)+
                }
            }
        }

        impl<'a> Readable<'a> for Opcode {
            fn read(reader: &mut Reader<'a>) -> Result<Self> {
                match reader.u8()? {
                    $($n => Ok(Opcode::$op),)+
                    _ => Err(ReadError::InvalidData),
                }
            }
        }

        impl Writable for Opcode {
            fn write(&self, out: &mut Vec<u8>) {
                out.push(*self as u8);
            }

            fn size(&self) -> usize {
                1
            }
        }

        impl<'a> Readable<'a> for Request<'a> {
            fn read(reader: &mut Reader<'a>) -> Result<Self> {
                let header = RequestHeader::read(reader)?;

                match header.major_opcode {
                    $(Opcode::$op => Ok(Request::$op($op::read(reader)?)),)+
                }
            }
        }

        impl<'a> Writable for Request<'a> {
            fn write(&self, out: &mut Vec<u8>) {
                let header = RequestHeader {
                    major_opcode: self.opcode(),
                    minor_opcode: 0,
                    size: (self.content_size() / 4) as u16,
                };

                header.write(out);

                match self {
                    $(Request::$op(req) => req.write(out),)+
                }
            }

            fn size(&self) -> usize {
                self.content_size() + 4
            }
        }
    };
}

define_request! {
    Connect = 1,
    ConnectReply = 2,
    // Disconnect = 3,
    // DisconnectReply = 4,
    //
    Open = 30,
    OpenReply = 31,
    // Close = 32,
    // CloseReply = 33,
    // RegisterTriggerKeys = 34,
    // TriggerNotifyReply = 36,
    //
    // SetEventMask = 37,
    // EncodingNegotiationReply = 39,
    // QueryExtensionReply = 41,
    // SetImValuesReply = 43,
    // GetImValuesReply = 45,
    //
    // CreateIcReply = 51,
    // DestroyIcReply = 53,
    // SetIcValuesReply = 55,
    // GetIcValuesReply = 57,
    // SyncReply = 62,
    // Commit = 63,
    // ResetIcReply = 65,
    //
    // Geometry = 70,
    // StrConversion = 71,
    // PreeditStart = 73,
    // PreeditDraw = 75,
    // PreeditCaret = 76,
    // PreeditDone = 78,
    // StatusStart = 79,
    // StatusDraw = 80,
    // StatusDone = 81,
    // PreeditState = 82,
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
#[repr(transparent)]
pub struct XimString<'a>(pub &'a [u8]);

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
#[repr(transparent)]
pub struct XimStr<'a>(pub &'a [u8]);

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct Connect<'a> {
    pub client_major_protocol_version: u16,
    pub client_minor_protocol_version: u16,
    pub client_auth_protocol_names: Vec<XimString<'a>>,
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub struct ConnectReply<'a> {
    pub server_major_protocol_version: u16,
    pub server_minor_protocol_version: u16,
    pub _marker: PhantomData<&'a ()>,
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub struct Open<'a> {
    pub name: XimStr<'a>,
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct OpenReply<'a> {
    pub input_method_id: u16,
    pub xim_attributes: Vec<Attr<'a>>,
    pub xic_attributes: Vec<Attr<'a>>,
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub struct Attr<'a> {
    pub id: u16,
    pub type_: u16,
    pub name: XimString<'a>,
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub struct Attribute<'a> {
    pub id: u16,
    pub name: XimString<'a>,
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub struct EncodingInfo<'a> {
    pub name: XimString<'a>,
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct StrConvText<'a> {
    pub type_: u16,
    pub text: XimString<'a>,
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub struct TriggerKey {
    pub keysym: u32,
    pub modifier: u32,
    pub modifier_mask: u32,
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub struct PreeditCaret {
    pub method_id: u16,
    pub context_id: u16,
    pub position: i32,
    pub direction: CaretDirection,
    pub style: CaretStyle,
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub struct PreeditCaretReply {
    pub method_id: u16,
    pub context_id: u16,
    pub position: i32,
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub struct PreeditDone {
    pub method_id: u16,
    pub context_id: u16,
}

#[derive(Eq, PartialEq, Copy, Clone, Debug, FromPrimitive)]
#[repr(u32)]
pub enum CaretDirection {
    ForwardChar = 0,
    BackwardChar = 1,
    ForwardWord = 2,
    BackwardWord = 3,
    CaretUp = 4,
    CaretDown = 5,
    NextLine = 6,
    PreviousLine = 7,
    LineStart = 8,
    LineEnd = 9,
    AbsolutePosition = 10,
    DontChange = 11,
}

#[derive(Eq, PartialEq, Copy, Clone, Debug, FromPrimitive)]
#[repr(u32)]
pub enum CaretStyle {
    Invisible = 0,
    Primary = 1,
    Secondary = 2,
}

#[derive(Eq, PartialEq, Copy, Clone, Debug, FromPrimitive, BitFlags)]
#[repr(u16)]
pub enum StrConvFeedbackType {
    LeftEdge = 0x1,
    RightEdge = 0x2,
    TopEdge = 0x4,
    BottomEdge = 0x8,
    Convealed = 0x10,
    Wrapped = 0x20,
}

#[derive(Eq, PartialEq, Copy, Clone, Debug, FromPrimitive, BitFlags)]
#[repr(u32)]
pub enum Feedback {
    Reverse = 0x1,
    Underline = 0x2,
    Highlight = 0x4,
    Primary = 0x8,
    Secondary = 0x10,
    Tertiary = 0x20,
    VisibleToForward = 0x40,
    VisibleToBackward = 0x80,
    VisibleToCenter = 0x100,
}

#[derive(Eq, PartialEq, Copy, Clone, Debug, FromPrimitive)]
#[repr(u32)]
pub enum HotkeyState {
    On = 0x1,
    Off = 0x2,
}

#[derive(Eq, PartialEq, Copy, Clone, Debug, FromPrimitive)]
#[repr(u32)]
pub enum PreeditState {
    Enable = 0x1,
    Disable = 0x2,
}

#[derive(Eq, PartialEq, Copy, Clone, Debug, FromPrimitive)]
#[repr(u32)]
pub enum ResetState {
    Initial = 0x1,
    Preserve = 0x2,
}
