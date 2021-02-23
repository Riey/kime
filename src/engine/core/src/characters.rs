use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use serde::{Deserialize, Serialize};
use std::{convert::TryFrom, str::FromStr};

use crate::{config::Addon, Config};

macro_rules! impl_jamo {
    ($ty:ty, [$(($item:ident, $ch:expr),)+]) => {
        impl $ty {
            pub const fn jamo(self) -> char {
                match self {
                    $(
                        Self::$item => $ch,
                    )+
                }
            }

            pub const fn from_jamo(c: char) -> Option<Self> {
                match c {
                    $(
                        $ch => Some(Self::$item),
                    )+
                    _ => None,
                }
            }
        }
    };
}

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

impl_jamo!(
    Choseong,
    [
        (Giyeok, 'ㄱ'),
        (SsangGiyeok, 'ㄲ'),
        (Nieun, 'ㄴ'),
        (Digeut, 'ㄷ'),
        (SsangDigeut, 'ㄸ'),
        (Rieul, 'ㄹ'),
        (Mieum, 'ㅁ'),
        (Bieup, 'ㅂ'),
        (SsangBieup, 'ㅃ'),
        (Siot, 'ㅅ'),
        (SsangSiot, 'ㅆ'),
        (Ieung, 'ㅇ'),
        (Jieut, 'ㅈ'),
        (SsangJieut, 'ㅉ'),
        (Chieut, 'ㅊ'),
        (Kiyeok, 'ㅋ'),
        (Tieut, 'ㅌ'),
        (Pieup, 'ㅍ'),
        (Hieuh, 'ㅎ'),
    ]
);
impl_jamo!(
    Jungseong,
    [
        (A, 'ㅏ'),
        (AE, 'ㅐ'),
        (YA, 'ㅑ'),
        (YAE, 'ㅒ'),
        (EO, 'ㅓ'),
        (E, 'ㅔ'),
        (YEO, 'ㅕ'),
        (YE, 'ㅖ'),
        (O, 'ㅗ'),
        (WA, 'ㅘ'),
        (WAE, 'ㅙ'),
        (OE, 'ㅚ'),
        (YO, 'ㅛ'),
        (U, 'ㅜ'),
        (WEO, 'ㅝ'),
        (WE, 'ㅞ'),
        (WI, 'ㅟ'),
        (YU, 'ㅠ'),
        (EU, 'ㅡ'),
        (YI, 'ㅢ'),
        (I, 'ㅣ'),
    ]
);
impl_jamo!(
    Jongseong,
    [
        (Giyeok, 'ㄱ'),
        (GiyeokSiot, 'ㄳ'),
        (SsangGiyeok, 'ㄲ'),
        (Nieun, 'ㄴ'),
        (NieunJieut, 'ㄵ'),
        (NieunHieuh, 'ㄶ'),
        (Digeut, 'ㄷ'),
        (Rieul, 'ㄹ'),
        (RieulGiyeok, 'ㄺ'),
        (RieulMieum, 'ㄻ'),
        (RieulBieup, 'ㄼ'),
        (RieulSiot, 'ㄽ'),
        (RieulTieut, 'ㄾ'),
        (RieulPieup, 'ㄿ'),
        (RieulHieuh, 'ㅀ'),
        (Mieum, 'ㅁ'),
        (Bieup, 'ㅂ'),
        (BieupSiot, 'ㅄ'),
        (Siot, 'ㅅ'),
        (SsangSiot, 'ㅆ'),
        (Ieung, 'ㅇ'),
        (Jieut, 'ㅈ'),
        (Chieut, 'ㅊ'),
        (Kieuk, 'ㅋ'),
        (Tieut, 'ㅌ'),
        (Pieup, 'ㅍ'),
        (Hieuh, 'ㅎ'),
    ]
);

impl Choseong {
    pub fn compose(self, jung: Jungseong, jong: Option<Jongseong>) -> char {
        unsafe {
            std::char::from_u32_unchecked(
                0xAC00 + self as u32 * 588 + jung as u32 * 28 + jong.map_or(0, |j| j as u32 + 1),
            )
        }
    }

    pub fn decompose(ch: char) -> Option<(Self, Jungseong, Option<Jongseong>)> {
        let n = ch as u32;
        let offset = n.checked_sub(0xAC00)?;
        let cho = FromPrimitive::from_u32(offset / 588)?;
        let offset = offset % 588;
        let jung = FromPrimitive::from_u32(offset / 28)?;
        let offset = offset % 28;

        let jong = match offset.checked_sub(1) {
            Some(offset) => Some(FromPrimitive::from_u32(offset)?),
            None => None,
        };

        Some((cho, jung, jong))
    }

    pub fn try_add(self, other: Self, config: &Config) -> Option<Self> {
        let compose_choseong_ssang = config.check_addon(Addon::ComposeChoseongSsang);
        match (self, other) {
            (Self::Giyeok, Self::Giyeok) if compose_choseong_ssang => Some(Self::SsangGiyeok),
            (Self::Bieup, Self::Bieup) if compose_choseong_ssang => Some(Self::SsangBieup),
            (Self::Siot, Self::Siot) if compose_choseong_ssang => Some(Self::SsangSiot),
            (Self::Jieut, Self::Jieut) if compose_choseong_ssang => Some(Self::SsangJieut),
            (Self::Digeut, Self::Digeut) if compose_choseong_ssang => Some(Self::SsangDigeut),
            _ => None,
        }
    }

    pub fn backspace(self, config: &Config) -> Option<Self> {
        let decompose_choseong_ssang = config.check_addon(Addon::DecomposeChoseongSsang);
        match self {
            Self::SsangGiyeok if decompose_choseong_ssang => Some(Self::Giyeok),
            Self::SsangBieup if decompose_choseong_ssang => Some(Self::Bieup),
            Self::SsangSiot if decompose_choseong_ssang => Some(Self::Siot),
            Self::SsangJieut if decompose_choseong_ssang => Some(Self::SsangJieut),
            Self::SsangDigeut if decompose_choseong_ssang => Some(Self::Digeut),
            _ => None,
        }
    }
}

impl Jungseong {
    pub fn try_add(self, other: Self, config: &Config) -> Option<Self> {
        let compose_jungseong_ssang = config.check_addon(Addon::ComposeJungseongSsang);
        match (self, other) {
            // ㅑ ㅣ = ㅒ
            (Self::YA, Self::I) if compose_jungseong_ssang => Some(Self::YAE),
            // ㅕ ㅣ = ㅖ
            (Self::YEO, Self::I) if compose_jungseong_ssang => Some(Self::YE),
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

    pub fn backspace(self, config: &Config) -> Option<Self> {
        let decompose_jungseong_ssang = config.check_addon(Addon::DecomposeJungseongSsang);

        match self {
            // ㅖ -> ㅕ
            Self::YE if decompose_jungseong_ssang => Some(Self::YEO),
            // ㅒ -> ㅑ
            Self::YAE if decompose_jungseong_ssang => Some(Self::YA),
            // ㅘ -> ㅗ
            Self::WA => Some(Self::O),
            // ㅚ -> ㅗ
            Self::OE => Some(Self::O),
            // ㅙ -> ㅗ
            Self::WAE => Some(Self::O),
            // ㅝ -> ㅜ
            Self::WEO => Some(Self::U),
            // ㅞ -> ㅜ
            Self::WE => Some(Self::U),
            // ㅟ -> ㅜ
            Self::WI => Some(Self::U),
            // ㅢ -> ㅡ
            Self::YI => Some(Self::EU),
            _ => None,
        }
    }
}

impl Jongseong {
    pub fn try_add(self, other: Self, config: &Config) -> Option<Self> {
        let compose_jongseong_ssang = config.check_addon(Addon::ComposeJongseongSsang);

        match (self, other) {
            (Self::Giyeok, Self::Giyeok) if compose_jongseong_ssang => Some(Self::SsangGiyeok),
            (Self::Siot, Self::Siot) if compose_jongseong_ssang => Some(Self::SsangSiot),

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
            _ => None,
        }
    }

    pub fn backspace(self, config: &Config) -> Option<Self> {
        let decompose_jongseong_ssang = config.check_addon(Addon::DecomposeJongseongSsang);

        match self {
            Self::SsangGiyeok if decompose_jongseong_ssang => Some(Self::Giyeok),
            Self::SsangSiot if decompose_jongseong_ssang => Some(Self::Siot),
            Self::GiyeokSiot => Some(Self::Giyeok),
            Self::NieunHieuh | Self::NieunJieut => Some(Self::Nieun),
            Self::RieulMieum
            | Self::RieulBieup
            | Self::RieulSiot
            | Self::RieulTieut
            | Self::RieulHieuh => Some(Self::Rieul),
            Self::BieupSiot => Some(Self::Bieup),
            _ => None,
        }
    }

    pub fn to_cho(self, config: &Config) -> JongToCho {
        let decompose_jongseong_ssang = config.check_addon(Addon::DecomposeJongseongSsang);

        use JongToCho::{Compose, Direct};
        match self {
            Self::Giyeok => Direct(Choseong::Giyeok),
            Self::SsangGiyeok if decompose_jongseong_ssang => {
                Compose(Self::Giyeok, Choseong::Giyeok)
            }
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
            Self::SsangSiot if decompose_jongseong_ssang => Compose(Self::Siot, Choseong::Siot),
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KeyValue {
    Choseong {
        cho: Choseong,
    },
    Jongseong {
        jong: Jongseong,
    },
    Jungseong {
        jung: Jungseong,
        compose: bool,
    },

    ChoJong {
        cho: Choseong,
        jong: Jongseong,
        first: bool,
    },

    ChoJung {
        cho: Choseong,
        jung: Jungseong,
        first: bool,
        compose: bool,
    },

    JungJong {
        jung: Jungseong,
        jong: Jongseong,
        first: bool,
        compose: bool,
    },

    Pass(Box<str>),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum KeyValuePart {
    /// Choseong
    Cho { cho: Choseong },
    /// Juneseong
    Jung { jung: Jungseong, compose: bool },
    /// Jongseong
    Jong { jong: Jongseong },
}

impl KeyValuePart {
    pub fn parse(chars: &mut std::str::Chars) -> Option<KeyValuePart> {
        match chars.next()? {
            '$' => {
                let next = chars.next()?;
                if let Some(jung) = Jungseong::from_jamo(next) {
                    Some(KeyValuePart::Jung {
                        jung,
                        compose: true,
                    })
                } else {
                    Some(KeyValuePart::Jong {
                        jong: Jongseong::from_jamo(next)?,
                    })
                }
            }
            c => {
                if let Some(cho) = Choseong::from_jamo(c) {
                    Some(Self::Cho { cho })
                } else {
                    Some(Self::Jung {
                        jung: Jungseong::from_jamo(c)?,
                        compose: false,
                    })
                }
            }
        }
    }
}

impl FromStr for KeyValue {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut chars = s.chars();

        let mut next = move || KeyValuePart::parse(&mut chars);

        match next() {
            None => Ok(Self::Pass(s.into())),
            Some(first) => match first {
                KeyValuePart::Cho { cho } => match next() {
                    Some(KeyValuePart::Cho { .. }) => Err(()),
                    Some(KeyValuePart::Jong { jong }) => Ok(Self::ChoJong {
                        cho,
                        jong,
                        first: true,
                    }),
                    Some(KeyValuePart::Jung { jung, compose }) => Ok(Self::ChoJung {
                        cho,
                        jung,
                        first: true,
                        compose,
                    }),
                    None => Ok(Self::Choseong { cho }),
                },
                KeyValuePart::Jung { jung, compose } => match next() {
                    Some(KeyValuePart::Cho { cho }) => Ok(Self::ChoJung {
                        cho,
                        jung,
                        first: false,
                        compose,
                    }),
                    Some(KeyValuePart::Jong { jong }) => Ok(Self::JungJong {
                        jung,
                        jong,
                        first: true,
                        compose,
                    }),
                    Some(KeyValuePart::Jung { .. }) => Err(()),
                    None => Ok(Self::Jungseong { jung, compose }),
                },
                KeyValuePart::Jong { jong } => match next() {
                    Some(KeyValuePart::Cho { cho }) => Ok(Self::ChoJong {
                        cho,
                        jong,
                        first: false,
                    }),
                    Some(KeyValuePart::Jong { .. }) => Err(()),
                    Some(KeyValuePart::Jung { jung, compose }) => Ok(Self::JungJong {
                        jung,
                        jong,
                        first: false,
                        compose,
                    }),
                    None => Ok(Self::Jongseong { jong }),
                },
            },
        }
    }
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

#[test]
fn decompose() {
    let (cho, jung, jong) = Choseong::decompose('앙').unwrap();
    assert_eq!('앙', cho.compose(jung, jong));
}

#[test]
fn parse_keyvalue() {
    assert_eq!(
        "ㅇ".parse::<KeyValue>().unwrap(),
        KeyValue::Choseong {
            cho: Choseong::Ieung
        }
    );
    assert_eq!(
        "$ㅇㅇ".parse::<KeyValue>().unwrap(),
        KeyValue::ChoJong {
            cho: Choseong::Ieung,
            jong: Jongseong::Ieung,
            first: false
        }
    );
    assert_eq!(
        "ㅇ$ㅇ".parse::<KeyValue>().unwrap(),
        KeyValue::ChoJong {
            cho: Choseong::Ieung,
            jong: Jongseong::Ieung,
            first: true
        }
    );
    assert_eq!(
        "ㅏ".parse::<KeyValue>().unwrap(),
        KeyValue::Jungseong {
            jung: Jungseong::A,
            compose: false
        }
    );
    assert_eq!(
        "$ㅏ".parse::<KeyValue>().unwrap(),
        KeyValue::Jungseong {
            jung: Jungseong::A,
            compose: true
        }
    );
    assert_eq!(
        "ㅢ$ㅅ".parse::<KeyValue>().unwrap(),
        KeyValue::JungJong {
            jung: Jungseong::YI,
            jong: Jongseong::Siot,
            first: true,
            compose: false
        },
    );
    assert_eq!(
        "$ㅅㅢ".parse::<KeyValue>().unwrap(),
        KeyValue::JungJong {
            jung: Jungseong::YI,
            jong: Jongseong::Siot,
            first: false,
            compose: false
        },
    );
}
