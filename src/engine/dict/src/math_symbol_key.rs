use std::ops::{BitOr, BitOrAssign};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Style(pub u8);

impl BitOr for Style {
    type Output = Style;
    fn bitor(self, rhs: Style) -> Style {
        Style(self.0 | rhs.0)
    }
}

impl BitOrAssign for Style {
    fn bitor_assign(&mut self, rhs: Style) {
        *self = *self | rhs;
    }
}

impl Style {
    pub const NONE: Style = Style(0);
    pub const SF: Style = Style(1 << 0);
    pub const BF: Style = Style(1 << 1);
    pub const IT: Style = Style(1 << 2);
    pub const TT: Style = Style(1 << 3);
    pub const BB: Style = Style(1 << 4);
    pub const SCR: Style = Style(1 << 5);
    pub const CAL: Style = Style(1 << 6);
    pub const FRAK: Style = Style(1 << 7);
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct SymbolKey<'a>(pub &'a str, pub Style);
