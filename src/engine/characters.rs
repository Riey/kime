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
    A = 0, // ㅏ
    AE,    // ㅐ
    YA,    // ㅑ
    YAE,   // ㅒ
    EO,    // ㅓ
    E,     // ㅔ
    YEO,   // ㅕ
    YE,    // ㅖ
    O,     // ㅗ
    WA,    // ㅘ
    WAE,   // ㅙ
    OE,    // ㅚ
    YO,    // ㅛ
    U,     // ㅜ
    WEO,   // ㅝ
    WE,    // ㅞ
    WI,    // ㅢ
    YU,    // ㅠ
    EU,    // ㅡ
    YI,    // 의
    I,     // ㅣ
}

impl_traits!(Choseong, 'ᄀ');
impl_traits!(Jungseong, 'ᅡ');
impl_traits!(Jongseong, 'ᆨ');

impl Choseong {
    pub fn compose(self, jung: Jungseong, jong: Option<Jongseong>) -> char {
        unsafe {
            std::char::from_u32_unchecked(
                0xAC00 + self as u32 * 588 + jung as u32 * 28 + jong.map_or(0, |j| j as u32),
            )
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
    pub fn jamo(self) -> char {
        match self {
            Self::A => 'ㅏ',
            Self::O => 'ㅗ',
            _ => todo!(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum KeyValue {
    Choseong(Choseong),
    Jungseong(Jungseong),
    Jongseong(Jongseong),
    // 두벌식용
    ChoJong(Choseong, Jongseong),
}

// impl Serialize for KeyValue {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: serde::Serializer,
//     {
//         match self {
//             KeyValue::Choseong(cho) => cho.serialize(serializer),
//             KeyValue::Jongseong(jong) => jong.serialize(serializer),
//             KeyValue::Jungseong(jung) => jung.serialize(serializer),
//             KeyValue::ChoJong(cho, jong) => {
//                 [char::from(*cho), char::from(*jong)].serialize(serializer)
//             }
//         }
//     }
// }
// struct KeyValueVisitor;

// impl<'de> Visitor<'de> for KeyValueVisitor {
//     type Value = KeyValue;

//     fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
//         formatter.write_str("char array")
//     }

//     fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
//     where
//         A: serde::de::SeqAccess<'de>,
//     {
//         use serde::de::Error;

//         let c: char = seq
//             .next_element()?
//             .ok_or_else(|| A::Error::custom("Empty"))?;

//         if let Ok(cho) = Choseong::try_from(c) {
//             match seq.next_element()? {
//                 Some(jong) => Ok(KeyValue::ChoJong(cho, jong)),
//                 None => Ok(KeyValue::Choseong(cho)),
//             }
//         } else if let Ok(jung) = Jungseong::try_from(c) {
//             Ok(KeyValue::Jungseong(jung))
//         } else if let Ok(jong) = Jongseong::try_from(c) {
//             Ok(KeyValue::Jongseong(jong))
//         } else {
//             Err(A::Error::invalid_value(Unexpected::Char(c), &self))
//         }
//     }
// }

// impl<'de> Deserialize<'de> for KeyValue {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//     where
//         D: serde::Deserializer<'de>,
//     {
//         deserializer.deserialize_seq(KeyValueVisitor)
//     }
// }
