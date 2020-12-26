use super::{
    compose::{compose_syllable, decompose_syllable},
    keycode::*,
    InputLayout, InputResult,
};

macro_rules! define_symbol {
    (
        @jaum [$(($jaum_key:tt, $jaum_ch:expr, $cho:expr, $jong:expr),)+]
        @moum [$(($moum_key:tt, $moum_ch:expr, $moum:expr),)+]
    ) => {
        fn jong_to_cho(jong: char) -> char {
            match jong {
                $(
                    $jong => $cho,
                )+
                _ => '\0',
            }
        }

        fn cho_to_char(cho: char) -> char {
            match cho {
                $(
                    $cho => $jaum_ch,
                )+
                _ => '\0',
            }
        }

        fn moum_to_char(mo: char) -> char {
            match mo {
                $(
                    $moum => $moum_ch,
                )+
                _ => '\0',
            }
        }

        impl InputLayout for DubeolSik {
            fn map_key(&mut self, keycode: u8) -> InputResult {
                match keycode {
                    $(
                        $jaum_key => self.state.jaum($cho, $jong),
                    )+
                    $(
                        $moum_key => self.state.moum($moum),
                    )+
                    _ => self.state.other(),
                }
            }
        }
    };
}

define_symbol! {
    @jaum [
        (Q, 'ㅂ', 'ᄇ', 'ᆸ'),
        (W, 'ㅈ', 'ᄌ', 'ᆽ'),
        (E, 'ㄷ', 'ᄃ', 'ᆮ'),
        (R, 'ㄱ', 'ᄀ', 'ᆨ'),
        (T, 'ㅅ', 'ᄉ', 'ᆺ'),
        (A, 'ㅁ', 'ᄆ', 'ᆷ'),
        (S, 'ㄴ', 'ᄂ', 'ᆫ'),
        (D, 'ㅇ', 'ᄋ', 'ᆼ'),
        (F, 'ㄹ', 'ᄅ', 'ᆯ'),
        (G, 'ㅎ', 'ᄒ', 'ᇂ'),
        (Z, 'ㅋ', 'ᄏ', 'ᆿ'),
        (X, 'ㅌ', 'ᄐ', 'ᇀ'),
        (C, 'ㅊ', 'ᄎ', 'ᆾ'),
        (V, 'ㅍ', 'ᄑ', 'ᇁ'),
    ]

    @moum [
        (Y, 'ㅛ', 'ᅭ'),
        (I, 'ㅛ', 'ᅣ'),
        (O, 'ㅐ', 'ᅢ'),
        (P, 'ㅔ', 'ᅦ'),

        (H, 'ㅗ', 'ᅩ'),
        (J, 'ㅓ', 'ᅥ'),
        (K, 'ㅏ', 'ᅡ'),
        (L, 'ㅣ', 'ᅵ'),

        (B, 'ㅠ', 'ᅲ'),
        (N, 'ㅜ', 'ᅮ'),
        (M, 'ㅡ', 'ᅳ'),
    ]
}

macro_rules! check_compose {
    ($prev_ch:expr, $current_ch:expr, $prev:expr, $current:expr, $out:expr) => {
        if $prev_ch == $prev && $current_ch == $current {
            return InputResult::
        }
    };
}

#[derive(Clone, Copy)]
enum DubeolSikState {
    Empty,
    Choseong(char),
    JungSeong(char),
    ChoseongJungSeong(char),
    Complete(char),
}

impl DubeolSikState {
    pub fn jaum(&mut self, choseong: char, jongseong: char) -> InputResult {
        match *self {
            DubeolSikState::Empty => {
                *self = DubeolSikState::Choseong(choseong);
                InputResult::Preedit(choseong)
            }
            DubeolSikState::Choseong(ch) => {
                *self = DubeolSikState::Choseong(choseong);
                InputResult::Commit(cho_to_char(ch))
            }
            DubeolSikState::JungSeong(ch) => {
                let ch = compose_syllable(choseong, ch).unwrap();
                *self = DubeolSikState::ChoseongJungSeong(ch);
                InputResult::Preedit(moum_to_char(ch))
            }
            DubeolSikState::ChoseongJungSeong(ch) => {
                let ch = compose_syllable(ch, jongseong).unwrap();
                *self = DubeolSikState::Complete(ch);
                InputResult::Preedit(ch)
            }
            DubeolSikState::Complete(ch) => {
                *self = DubeolSikState::Choseong(choseong);
                InputResult::Commit(ch)
            }
        }
    }

    pub fn moum(&mut self, jungseong: char) -> InputResult {
        match *self {
            DubeolSikState::Empty => {
                *self = DubeolSikState::JungSeong(jungseong);
                InputResult::Preedit(jungseong)
            }
            DubeolSikState::Choseong(ch) => {
                let ch = compose_syllable(ch, jungseong).unwrap();
                *self = DubeolSikState::ChoseongJungSeong(ch);
                InputResult::Preedit(ch)
            }
            DubeolSikState::JungSeong(ch) => {
                *self = DubeolSikState::JungSeong(jungseong);
                InputResult::Commit(moum_to_char(ch))
            }
            DubeolSikState::ChoseongJungSeong(ch) => {
                *self = DubeolSikState::JungSeong(jungseong);
                InputResult::Commit(ch)
            }
            DubeolSikState::Complete(ch) => {
                let (cho, jung, jong) = decompose_syllable(ch);

                debug_assert_ne!(jong, '\0');

                dbg!(jong);
                dbg!(jungseong);

                *self = DubeolSikState::ChoseongJungSeong(
                    compose_syllable(jong_to_cho(jong), jungseong).unwrap(),
                );

                InputResult::Commit(compose_syllable(cho, jung).unwrap())
            }
        }
    }

    pub fn other(&mut self) -> InputResult {
        match *self {
            DubeolSikState::Empty => InputResult::Bypass,
            DubeolSikState::Choseong(ch) | DubeolSikState::JungSeong(ch) => {
                *self = DubeolSikState::Empty;
                InputResult::CommitBypass(moum_to_char(ch))
            }
            DubeolSikState::ChoseongJungSeong(ch) | DubeolSikState::Complete(ch) => {
                *self = DubeolSikState::Empty;
                InputResult::CommitBypass(ch)
            }
        }
    }
}

pub struct DubeolSik {
    state: DubeolSikState,
}

impl DubeolSik {
    pub fn new() -> Self {
        Self {
            state: DubeolSikState::Empty,
        }
    }
}

#[test]
fn jo_to_cho() {
    assert_eq!(jong_to_cho('ᆺ'), 'ᄉ');
}
