use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KeyCode {
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Zero,
    Minus,
    Equal,
    Backslash,

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

    Bs,
    Henkan,
    Ralt,
}

impl KeyCode {
    pub fn from_x11_code(code: u8) -> Option<Self> {
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

            22 => Some(Self::Bs),

            100 => Some(Self::Henkan),
            108 => Some(Self::Ralt),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Key {
    pub code: KeyCode,
    pub shift: bool,
}
