use std::ops::BitOr;

#[derive(Debug,PartialEq,Eq,PartialOrd,Ord,Clone,Copy)]
pub struct Style(pub u8);

impl BitOr for Style {
    type Output = Style;
    fn bitor(self, rhs: Style) -> Style {
        Style(self.0 | rhs.0)
    }
}

pub const STYLE_NONE: Style = Style(0);
pub const STYLE_SF: Style = Style(1);
pub const STYLE_BF: Style = Style(2);
pub const STYLE_IT: Style = Style(4);
pub const STYLE_TT: Style = Style(8);
pub const STYLE_BB: Style = Style(16);
pub const STYLE_SCR: Style = Style(32);
pub const STYLE_CAL: Style = Style(64);
pub const STYLE_FRAK: Style = Style(128);

#[derive(Debug,PartialEq,Eq,PartialOrd,Ord,Clone,Copy)]
pub struct SymbolKey<'a>(pub &'a str, pub Style);
