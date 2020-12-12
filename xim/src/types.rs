use enumflags2::BitFlags;
use num_derive::FromPrimitive;

pub type C8 = u8;
pub type C16 = u16;
pub type C32 = u32;
pub type C64 = u64;
pub type CHAR = u8;
pub type WINDOW = C32;

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub struct Attr<'a> {
    pub id: C16,
    pub type_: C16,
    pub name: &'a str,
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub struct Attribute<'a> {
    pub id: C16,
    pub value: &'a str,
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub struct EncodingInfo<'a> {
    pub name: &'a str,
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct StrConvText<'a> {
    pub type_: C16,
    pub text: &'a str,
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub struct TriggerKey {
    pub keysym: C32,
    pub modifier: C32,
    pub modifier_mask: C32,
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub struct Extension<'a> {
    pub major_opcode: C8,
    pub minor_opcode: C8,
    pub name: &'a str,
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub struct PreeditCaret {
    pub method_id: C16,
    pub context_id: C16,
    pub position: i32,
    pub direction: CaretDirection,
    pub style: CaretStyle,
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub struct PreeditCaretReply {
    pub method_id: C16,
    pub context_id: C16,
    pub position: i32,
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub struct PreeditDone {
    pub method_id: C16,
    pub context_id: C16,
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
