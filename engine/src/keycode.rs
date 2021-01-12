use std::str::FromStr;

use serde::{
    de::{Error, Unexpected},
    Deserialize,
};
use strum_macros::EnumString;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, EnumString)]
pub enum KeyCode {
    #[strum(to_string = "1")]
    One,
    #[strum(to_string = "2")]
    Two,
    #[strum(to_string = "3")]
    Three,
    #[strum(to_string = "4")]
    Four,
    #[strum(to_string = "5")]
    Five,
    #[strum(to_string = "6")]
    Six,
    #[strum(to_string = "7")]
    Seven,
    #[strum(to_string = "8")]
    Eight,
    #[strum(to_string = "9")]
    Nine,
    #[strum(to_string = "0")]
    Zero,

    Minus,
    Equal,
    Backslash,
    Space,

    Q,
    W,
    E,
    R,
    T,
    Y,
    U,
    I,
    O,
    P,

    A,
    S,
    D,
    F,
    G,
    H,
    J,
    K,
    L,

    Z,
    X,
    C,
    V,
    B,
    N,
    M,

    LeftArrow,
    RightArrow,
    UpArrow,
    DownArrow,

    Esc,
    Backspace,
    Henkan,
    AltR,
    Hangul,
}

impl KeyCode {
    pub const fn from_hardward_code(code: u8) -> Option<Self> {
        match code {
            10 => Some(Self::One),
            11 => Some(Self::Two),
            12 => Some(Self::Three),
            13 => Some(Self::Four),
            14 => Some(Self::Five),
            15 => Some(Self::Six),
            16 => Some(Self::Seven),
            17 => Some(Self::Eight),
            18 => Some(Self::Nine),
            19 => Some(Self::Zero),
            20 => Some(Self::Minus),
            21 => Some(Self::Equal),
            51 => Some(Self::Backslash),

            24 => Some(Self::Q),
            25 => Some(Self::W),
            26 => Some(Self::E),
            27 => Some(Self::R),
            28 => Some(Self::T),
            29 => Some(Self::Y),
            30 => Some(Self::U),
            31 => Some(Self::I),
            32 => Some(Self::O),
            33 => Some(Self::P),

            38 => Some(Self::A),
            39 => Some(Self::S),
            40 => Some(Self::D),
            41 => Some(Self::F),
            42 => Some(Self::G),
            43 => Some(Self::H),
            44 => Some(Self::J),
            45 => Some(Self::K),
            46 => Some(Self::L),

            52 => Some(Self::Z),
            53 => Some(Self::X),
            54 => Some(Self::C),
            55 => Some(Self::V),
            56 => Some(Self::B),
            57 => Some(Self::N),
            58 => Some(Self::M),

            22 => Some(Self::Backspace),
            65 => Some(Self::Space),

            113 => Some(Self::LeftArrow),
            114 => Some(Self::RightArrow),
            111 => Some(Self::UpArrow),
            116 => Some(Self::DownArrow),

            9 => Some(Self::Esc),
            100 => Some(Self::Henkan),
            108 => Some(Self::AltR),
            // 65329 => Some(Self::Hangul),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Key {
    pub code: KeyCode,
    pub shift: bool,
}

impl Key {
    pub const fn new(code: KeyCode, shift: bool) -> Self {
        Self { code, shift }
    }

    pub const fn normal(code: KeyCode) -> Self {
        Self::new(code, false)
    }

    pub const fn shift(code: KeyCode) -> Self {
        Self::new(code, true)
    }
}

impl FromStr for Key {
    type Err = <KeyCode as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(s) = s.strip_prefix("S-") {
            Ok(Self::new(s.parse()?, true))
        } else {
            Ok(Self::new(s.parse()?, false))
        }
    }
}

impl<'de> Deserialize<'de> for Key {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        Self::from_str(&s).map_err(|_e| D::Error::invalid_value(Unexpected::Str(&s), &"Key"))
    }
}
