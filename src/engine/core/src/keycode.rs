use std::{fmt, str::FromStr};

use serde::{
    de::{Error, Unexpected},
    Deserialize, Serialize,
};
use strum_macros::{Display, EnumString};

bitflags::bitflags! {
    #[repr(transparent)]
    pub struct ModifierState: u32 {
        const CONTROL = 0x1;
        const SUPER = 0x2;
        const SHIFT = 0x4;
        const ALT = 0x8;
    }
}

// TODO: complete
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, EnumString, Display)]
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
    Grave,
    OpenBracket,
    CloseBracket,
    Space,

    Comma,
    Period,
    SemiColon,
    Quote,
    Slash,

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

    Esc,
    Shift,
    Backspace,
    Muhenkan,
    Henkan,
    AltR,
    Hangul,
}

impl KeyCode {
    pub const fn from_hardward_code(code: u16) -> Option<Self> {
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
            34 => Some(Self::OpenBracket),
            35 => Some(Self::CloseBracket),
            // Shift_L, Shift_R
            50 | 62 => Some(Self::Shift),
            51 => Some(Self::Backslash),
            61 => Some(Self::Slash),
            47 => Some(Self::SemiColon),
            48 => Some(Self::Quote),
            49 => Some(Self::Grave),
            59 => Some(Self::Comma),
            60 => Some(Self::Period),

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

            9 => Some(Self::Esc),
            100 => Some(Self::Henkan),
            102 => Some(Self::Muhenkan),
            108 => Some(Self::AltR),
            122 => Some(Self::Hangul),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Key {
    pub code: KeyCode,
    pub state: ModifierState,
}

impl Key {
    pub const fn new(code: KeyCode, state: ModifierState) -> Self {
        Self { code, state }
    }

    pub const fn normal(code: KeyCode) -> Self {
        Self::new(code, ModifierState::empty())
    }

    pub const fn shift(code: KeyCode) -> Self {
        Self::new(code, ModifierState::SHIFT)
    }

    pub const fn alt(code: KeyCode) -> Self {
        Self::new(code, ModifierState::ALT)
    }

    pub const fn ctrl(code: KeyCode) -> Self {
        Self::new(code, ModifierState::CONTROL)
    }

    pub const fn super_(code: KeyCode) -> Self {
        Self::new(code, ModifierState::SUPER)
    }
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.state.contains(ModifierState::SUPER) {
            f.write_str("Super-")?;
        }

        if self.state.contains(ModifierState::ALT) {
            f.write_str("M-")?;
        }

        if self.state.contains(ModifierState::CONTROL) {
            f.write_str("C-")?;
        }

        if self.state.contains(ModifierState::SHIFT) {
            f.write_str("S-")?;
        }

        write!(f, "{}", self.code)
    }
}

impl FromStr for Key {
    type Err = <KeyCode as FromStr>::Err;

    fn from_str(mut s: &str) -> Result<Self, Self::Err> {
        let mut state = ModifierState::empty();

        loop {
            if let Some(n) = s.strip_prefix("Super-") {
                s = n;
                state |= ModifierState::SUPER;
                continue;
            }

            if let Some(n) = s.strip_prefix("M-") {
                s = n;
                state |= ModifierState::ALT;
                continue;
            }

            if let Some(n) = s.strip_prefix("C-") {
                s = n;
                state |= ModifierState::CONTROL;
                continue;
            }

            if let Some(n) = s.strip_prefix("S-") {
                s = n;
                state |= ModifierState::SHIFT;
                continue;
            }

            break;
        }

        Ok(Self::new(s.parse()?, state))
    }
}

impl Serialize for Key {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = self.to_string();
        serializer.serialize_str(&s)
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

#[test]
fn key_parse() {
    assert_eq!(
        "Super-Space".parse::<Key>().unwrap(),
        Key::super_(KeyCode::Space)
    );
    assert_eq!("S-4".parse::<Key>().unwrap(), Key::shift(KeyCode::Four));
    assert_eq!("C-Space".parse::<Key>().unwrap(), Key::ctrl(KeyCode::Space));
    assert_eq!("M-X".parse::<Key>().unwrap(), Key::alt(KeyCode::X));
}
