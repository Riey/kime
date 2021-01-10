use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

macro_rules! impl_traits {
    ($ty:ty, $first_ch:expr) => {
        impl std::fmt::Display for $ty {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "{}", char::from(*self))
            }
        }

        impl From<$ty> for char {
            fn from(c: $ty) -> char {
                unsafe { std::char::from_u32_unchecked($first_ch as u32 + c as u32) }
            }
        }

        impl TryFrom<char> for $ty {
            type Error = ();

            fn try_from(ch: char) -> Result<Self, Self::Error> {
                use std::convert::TryInto;

                (ch as u32).try_into()
            }
        }

        impl TryFrom<u32> for $ty {
            type Error = ();

            fn try_from(n: u32) -> Result<Self, Self::Error> {
                match n.checked_sub($first_ch as u32) {
                    Some(idx) => FromPrimitive::from_u32(idx).ok_or(()),
                    _ => Err(()),
                }
            }
        }

        impl Serialize for $ty {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                serializer.serialize_char((*self).into())
            }
        }

        impl<'de> Deserialize<'de> for $ty {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                use serde::de::Error;
                use std::convert::TryInto;

                let ch = char::deserialize(deserializer)?;

                ch.try_into()
                    .map_err(|_| D::Error::custom(concat!("Not ", stringify!($ty))))
            }
        }
    };
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, FromPrimitive)]
#[repr(u32)]
pub enum Choseong {
    Giyeok = 0,
    SsangGiyeok,
    Nieun,
    Digeut,
    SsangDigeut,
    Rieul,
    Mieum,
    Bieup,
    SsangBieup,
    Siot,
    SsangSiot,
    Ieung,
    Jieut,
    SsangJieut,
    Chieut,
    Kiyeok,
    Tieut,
    Pieup,
    Hieuh,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, FromPrimitive)]
#[repr(u32)]
pub enum Jongseong {
    Giyeok = 0,
    SsangGiyeok,
    GiyeokSiot,
    Nieun,
    NieunJieut,
    NieunHieuh,
    Digeut,
    Rieul,
    RieulGiyeok,
    RieulMieum,
    RieulBieup,
    RieulSiot,
    RieulTieut,
    RieulPieup,
    RieulHieuh,
    Mieum,
    Bieup,
    BieupSiot,
    Siot,
    SsangSiot,
    Ieung,
    Jieut,
    Chieut,
    Kieuk,
    Tieut,
    Pieup,
    Hieuh,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, FromPrimitive)]
#[repr(u32)]
pub enum Jungseong {
    A = 0,
    AE,
    YA,
    YAE,
    EO,
    E,
    YEO,
    YE,
    O,
    WA,
    WAE,
    OE,
    YO,
    U,
    WEO,
    WE,
    WI,
    YU,
    EU,
    YI,
    I,
}

impl_traits!(Choseong, 'ᄀ');
impl_traits!(Jungseong, 'ᅡ');
impl_traits!(Jongseong, 'ᆨ');

impl Choseong {
    pub fn compose(self, jung: Jungseong, jong: Option<Jongseong>) -> char {
        unsafe {
            std::char::from_u32_unchecked(
                0xAC00 + self as u32 * 588 + jung as u32 * 28 + jong.map_or(0, |j| j as u32 + 1),
            )
        }
    }

    pub const fn try_add(self, other: Self) -> Option<Self> {
        match (self, other) {
            (Self::Giyeok, Self::Giyeok) => Some(Self::SsangGiyeok),
            (Self::Bieup, Self::Bieup) => Some(Self::SsangBieup),
            (Self::Siot, Self::Siot) => Some(Self::SsangSiot),
            (Self::Jieut, Self::Jieut) => Some(Self::SsangJieut),
            (Self::Digeut, Self::Digeut) => Some(Self::SsangDigeut),
            _ => None,
        }
    }

    pub const fn backspace(self) -> Option<Self> {
        match self {
            Self::SsangGiyeok => Some(Self::Giyeok),
            Self::SsangBieup => Some(Self::Bieup),
            Self::SsangSiot => Some(Self::Siot),
            Self::SsangJieut => Some(Self::SsangJieut),
            Self::SsangDigeut => Some(Self::Digeut),
            _ => None,
        }
    }

    pub const fn jamo(self) -> char {
        match self {
            Self::Giyeok => 'ㄱ',
            Self::SsangGiyeok => 'ㄲ',
            Self::Nieun => 'ㄴ',
            Self::Digeut => 'ㄷ',
            Self::SsangDigeut => 'ㄸ',
            Self::Rieul => 'ㄹ',
            Self::Mieum => 'ㅁ',
            Self::Bieup => 'ㅂ',
            Self::SsangBieup => 'ㅃ',
            Self::Siot => 'ㅅ',
            Self::SsangSiot => 'ㅆ',
            Self::Ieung => 'ㅇ',
            Self::Jieut => 'ㅈ',
            Self::SsangJieut => 'ㅉ',
            Self::Chieut => 'ㅊ',
            Self::Kiyeok => 'ㅋ',
            Self::Tieut => 'ㅌ',
            Self::Pieup => 'ㅍ',
            Self::Hieuh => 'ㅎ',
        }
    }
}

impl Jungseong {
    pub const fn jamo(self) -> char {
        match self {
            Self::A => 'ㅏ',
            Self::AE => 'ㅐ',
            Self::YA => 'ㅑ',
            Self::YAE => 'ㅒ',
            Self::EO => 'ㅓ',
            Self::E => 'ㅔ',
            Self::YEO => 'ㅕ',
            Self::YE => 'ㅖ',
            Self::O => 'ㅗ',
            Self::WA => 'ㅘ',
            Self::WAE => 'ㅙ',
            Self::OE => 'ㅚ',
            Self::YO => 'ㅛ',
            Self::U => 'ㅜ',
            Self::WEO => 'ㅝ',
            Self::WE => 'ㅞ',
            Self::WI => 'ㅟ',
            Self::YU => 'ㅠ',
            Self::EU => 'ㅡ',
            Self::YI => 'ㅢ',
            Self::I => 'ㅣ',
        }
    }

    pub const fn try_add(self, other: Self) -> Option<Self> {
        match (self, other) {
            // ㅑ ㅣ = ㅒ
            (Self::YA, Self::I) => Some(Self::YAE),
            // ㅕ ㅣ = ㅖ
            (Self::YEO, Self::I) => Some(Self::YE),
            // ㅗ ㅏ = ㅘ
            (Self::O, Self::A) => Some(Self::WA),
            // ㅗ ㅣ = ㅚ
            (Self::O, Self::I) => Some(Self::OE),
            // ㅗ ㅐ = ㅙ
            (Self::O, Self::AE) => Some(Self::WAE),
            // ㅜ ㅓ = ㅝ
            (Self::U, Self::EO) => Some(Self::WEO),
            // ㅜ ㅔ = ㅞ
            (Self::U, Self::E) => Some(Self::WE),
            // ㅜ ㅣ = ㅟ
            (Self::U, Self::I) => Some(Self::WI),
            // ㅡ ㅣ = ㅢ
            (Self::EU, Self::I) => Some(Self::YI),
            _ => None,
        }
    }

    pub const fn backspace(self) -> Option<Self> {
        match self {
            Self::AE => Some(Self::A),
            Self::YAE => Some(Self::YA),
            Self::WA | Self::OE => Some(Self::O),
            _ => None,
        }
    }
}

impl Jongseong {
    pub const fn jamo(self) -> char {
        match self {
            Self::Giyeok => 'ㄱ',
            Self::GiyeokSiot => 'ㄳ',
            Self::SsangGiyeok => 'ㄲ',
            Self::Nieun => 'ㄴ',
            Self::NieunJieut => 'ㄵ',
            Self::NieunHieuh => 'ㄶ',
            Self::Digeut => 'ㄷ',
            Self::Rieul => 'ㄹ',
            Self::RieulGiyeok => 'ㄺ',
            Self::RieulMieum => 'ㄻ',
            Self::RieulBieup => 'ㄼ',
            Self::RieulSiot => 'ㄽ',
            Self::RieulTieut => 'ㄾ',
            Self::RieulPieup => 'ㄿ',
            Self::RieulHieuh => 'ㅀ',
            Self::Mieum => 'ㅁ',
            Self::Bieup => 'ㅂ',
            Self::BieupSiot => 'ㅄ',
            Self::Siot => 'ㅅ',
            Self::SsangSiot => 'ㅆ',
            Self::Ieung => 'ㅇ',
            Self::Jieut => 'ㅈ',
            Self::Chieut => 'ㅊ',
            Self::Kieuk => 'ㅋ',
            Self::Tieut => 'ㅌ',
            Self::Pieup => 'ㅍ',
            Self::Hieuh => 'ㅎ',
        }
    }

    pub const fn try_add(self, other: Self) -> Option<Self> {
        match (self, other) {
            (Self::Giyeok, Self::Giyeok) => Some(Self::SsangGiyeok),
            (Self::Giyeok, Self::Siot) => Some(Self::GiyeokSiot),
            (Self::Nieun, Self::Hieuh) => Some(Self::NieunHieuh),
            (Self::Nieun, Self::Jieut) => Some(Self::NieunJieut),
            (Self::Rieul, Self::Giyeok) => Some(Self::RieulGiyeok),
            (Self::Rieul, Self::Mieum) => Some(Self::RieulMieum),
            (Self::Rieul, Self::Bieup) => Some(Self::RieulBieup),
            (Self::Rieul, Self::Siot) => Some(Self::RieulSiot),
            (Self::Rieul, Self::Tieut) => Some(Self::RieulTieut),
            (Self::Rieul, Self::Pieup) => Some(Self::RieulPieup),
            (Self::Rieul, Self::Hieuh) => Some(Self::RieulHieuh),
            (Self::Bieup, Self::Siot) => Some(Self::BieupSiot),
            (Self::Siot, Self::Siot) => Some(Self::SsangSiot),
            _ => None,
        }
    }

    pub const fn backspace(self) -> Option<Self> {
        match self {
            Self::SsangGiyeok | Self::GiyeokSiot => Some(Self::Giyeok),
            Self::NieunHieuh | Self::NieunJieut => Some(Self::Nieun),
            Self::RieulMieum
            | Self::RieulBieup
            | Self::RieulSiot
            | Self::RieulTieut
            | Self::RieulHieuh => Some(Self::Rieul),
            Self::BieupSiot => Some(Self::Bieup),
            Self::SsangSiot => Some(Self::Siot),
            _ => None,
        }
    }

    pub const fn to_cho(self) -> JongToCho {
        use JongToCho::{Compose, Direct};
        match self {
            Self::Giyeok => Direct(Choseong::Giyeok),
            Self::SsangGiyeok => Direct(Choseong::SsangGiyeok),
            Self::GiyeokSiot => Compose(Self::Giyeok, Choseong::Siot),
            Self::Nieun => Direct(Choseong::Nieun),
            Self::NieunJieut => Compose(Self::Nieun, Choseong::Jieut),
            Self::NieunHieuh => Compose(Self::Nieun, Choseong::Hieuh),
            Self::Digeut => Direct(Choseong::Digeut),
            Self::Rieul => Direct(Choseong::Rieul),
            Self::RieulGiyeok => Compose(Self::Rieul, Choseong::Giyeok),
            Self::RieulMieum => Compose(Self::Rieul, Choseong::Mieum),
            Self::RieulBieup => Compose(Self::Rieul, Choseong::Bieup),
            Self::RieulSiot => Compose(Self::Rieul, Choseong::Siot),
            Self::RieulTieut => Compose(Self::Rieul, Choseong::Tieut),
            Self::RieulPieup => Compose(Self::Rieul, Choseong::Pieup),
            Self::RieulHieuh => Compose(Self::Rieul, Choseong::Hieuh),
            Self::Mieum => Direct(Choseong::Mieum),
            Self::Bieup => Direct(Choseong::Bieup),
            Self::BieupSiot => Compose(Self::Bieup, Choseong::Siot),
            Self::Siot => Direct(Choseong::Siot),
            Self::SsangSiot => Direct(Choseong::SsangSiot),
            Self::Ieung => Direct(Choseong::Ieung),
            Self::Jieut => Direct(Choseong::Jieut),
            Self::Chieut => Direct(Choseong::Chieut),
            Self::Kieuk => Direct(Choseong::Kiyeok),
            Self::Tieut => Direct(Choseong::Tieut),
            Self::Pieup => Direct(Choseong::Pieup),
            Self::Hieuh => Direct(Choseong::Hieuh),
        }
    }
}

#[derive(Clone, Copy)]
pub enum JongToCho {
    Direct(Choseong),
    Compose(Jongseong, Choseong),
}

#[derive(Clone, Copy, Debug)]
pub enum KeyValue {
    // 세벌식용
    Choseong(Choseong),
    Jongseong(Jongseong),

    // 두벌식용
    ChoJong(Choseong, Jongseong),

    Jungseong(Jungseong),

    // 한글이 아닌 문자
    Pass(char),
}

#[test]
fn compose() {
    assert_eq!('ㅇ', Jongseong::Ieung.jamo());
    assert_eq!(
        '앙',
        Choseong::Ieung.compose(Jungseong::A, Some(Jongseong::Ieung))
    );
    assert_eq!('아', Choseong::Ieung.compose(Jungseong::A, None));
}
