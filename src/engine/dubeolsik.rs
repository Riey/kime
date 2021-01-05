use super::{
    compose::{compose_syllable, decompose_syllable},
    keycode::*,
    InputLayout, InputResult,
};

macro_rules! define_symbol {
    (
        @jaum [$(($jaum_key:tt, $jaum_ch:expr, $cho:expr, $jong:expr),)+]
        @jong_compose [$(($jong_com:expr, $jong_com_left:expr, $jong_com_right:expr),)+]
        @moum [$(($moum_key:tt, $moum_ch:expr, $moum:expr),)+]
        @moum_compose [$(($moum_com_ch:expr, $moum_com:expr, $moum_com_left:expr, $moum_com_right:expr),)+]
    ) => {
        fn jong_direct_to_cho(jong: char) -> char {
            match jong {
                $(
                    $jong => $cho,
                )+
                _ => '\0',
            }
        }

        fn jong_to_cho(jong: char) -> (char, char) {
            match jong_direct_to_cho(jong) {
                '\0' => match jong {
                    $(
                        $jong_com => (jong_direct_to_cho($jong_com_left), jong_direct_to_cho($jong_com_right)),
                    )+
                    _ => ('\0', '\0'),
                }
                cho => (cho, '\0'),
            }
        }

        fn try_compose_jong(left: char, right: char) -> char {
            match (left, right) {
                $(
                    ($jong_com_left, $jong_com_right) => $jong_com,
                )+
                _ => '\0',
            }
        }

        fn try_decompose_jong(jong: char) -> (char, char) {
            match jong {
                $(
                    $jong_com => ($jong_com_left, $jong_com_right),
                )+
                _ => ('\0', '\0'),
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
                $(
                    $moum_com => $moum_com_ch,
                )+
                _ => '\0',
            }
        }

        fn try_compose_moum(left: char, right: char) -> char {
            match (left, right) {
                $(
                    ($moum_com_left, $moum_com_right) => $moum_com,
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
                    BS => self.state.backspace(),
                    _ => self.state.other(),
                }
            }

            fn reset(&mut self) -> Option<char> {
                self.state.reset()
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

    @jong_compose [
        ('ᆭ', 'ᆫ', 'ᇂ'),
        ('ᆹ', 'ᆸ', 'ᆺ'),
    ]

    @moum [
        (Y, 'ㅛ', 'ᅭ'),
        (I, 'ㅛ', 'ᅣ'),
        (O, 'ㅐ', 'ᅢ'),
        (U, 'ㅕ', 'ᅧ'),
        (P, 'ㅔ', 'ᅦ'),

        (H, 'ㅗ', 'ᅩ'),
        (J, 'ㅓ', 'ᅥ'),
        (K, 'ㅏ', 'ᅡ'),
        (L, 'ㅣ', 'ᅵ'),

        (B, 'ㅠ', 'ᅲ'),
        (N, 'ㅜ', 'ᅮ'),
        (M, 'ㅡ', 'ᅳ'),
    ]

    @moum_compose [
        ('ㅚ', 'ᅬ', 'ᅩ', 'ᅵ'),
    ]
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
    pub fn reset(&mut self) -> Option<char> {
        match *self {
            DubeolSikState::Empty => None,
            DubeolSikState::ChoseongJungSeong(ch) | DubeolSikState::Complete(ch) => Some(ch),
            DubeolSikState::Choseong(ch) => Some(cho_to_char(ch)),
            DubeolSikState::JungSeong(ch) => Some(moum_to_char(ch)),
        }
    }

    pub fn jaum(&mut self, choseong: char, jongseong: char) -> InputResult {
        match *self {
            DubeolSikState::Empty => {
                *self = DubeolSikState::Choseong(choseong);
                InputResult::Preedit(cho_to_char(choseong))
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
                let (cho, jung, jong) = decompose_syllable(ch);
                debug_assert_ne!(jong, '\0');

                let com_jong = try_compose_jong(jong, jongseong);

                match com_jong {
                    '\0' => {
                        *self = DubeolSikState::Choseong(choseong);
                        InputResult::CommitPreedit(ch, cho_to_char(choseong))
                    }
                    _ => {
                        let ch = compose_syllable(compose_syllable(cho, jung).unwrap(), com_jong)
                            .unwrap();
                        *self = DubeolSikState::Complete(ch);
                        InputResult::Preedit(ch)
                    }
                }
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
                let com = try_compose_moum(ch, jungseong);

                // compose failed
                if com == '\0' {
                    *self = DubeolSikState::JungSeong(jungseong);
                    InputResult::Commit(moum_to_char(ch))
                } else {
                    // 'ㅗ' + 'ㅣ' = 'ㅚ'
                    *self = DubeolSikState::Complete(com);
                    InputResult::Preedit(moum_to_char(com))
                }
            }
            DubeolSikState::ChoseongJungSeong(ch) => {
                let (cho, jung, _) = decompose_syllable(ch);

                let com = try_compose_moum(jung, jungseong);

                if com == '\0' {
                    *self = DubeolSikState::JungSeong(jungseong);
                    InputResult::Commit(ch)
                } else {
                    let ch = compose_syllable(cho, com).unwrap();
                    *self = DubeolSikState::ChoseongJungSeong(ch);
                    InputResult::Preedit(ch)
                }
            }
            DubeolSikState::Complete(ch) => {
                let (cho, jung, jong) = decompose_syllable(ch);

                debug_assert_ne!(jong, '\0');

                let (jong_left, jong_right) = jong_to_cho(jong);

                if jong_right != '\0' {
                    let preedit =
                        compose_syllable(jong_direct_to_cho(jong_right), jungseong).unwrap();
                    *self = DubeolSikState::ChoseongJungSeong(preedit);
                    InputResult::CommitPreedit(
                        compose_syllable(compose_syllable(cho, jung).unwrap(), jong_left).unwrap(),
                        preedit,
                    )
                } else {
                    let preedit = compose_syllable(jong_direct_to_cho(jong), jungseong).unwrap();
                    *self = DubeolSikState::ChoseongJungSeong(preedit);
                    InputResult::CommitPreedit(compose_syllable(cho, jung).unwrap(), preedit)
                }
            }
        }
    }

    pub fn other(&mut self) -> InputResult {
        match *self {
            DubeolSikState::Empty => InputResult::Bypass,
            DubeolSikState::Choseong(ch) => {
                *self = DubeolSikState::Empty;
                InputResult::CommitBypass(cho_to_char(ch))
            }
            DubeolSikState::JungSeong(ch) => {
                *self = DubeolSikState::Empty;
                InputResult::CommitBypass(moum_to_char(ch))
            }
            DubeolSikState::ChoseongJungSeong(ch) | DubeolSikState::Complete(ch) => {
                *self = DubeolSikState::Empty;
                InputResult::CommitBypass(ch)
            }
        }
    }

    pub fn backspace(&mut self) -> InputResult {
        match *self {
            DubeolSikState::Empty => InputResult::Bypass,
            DubeolSikState::Choseong(..) | DubeolSikState::JungSeong(..) => {
                *self = DubeolSikState::Empty;
                InputResult::ClearPreedit
            }
            // 가 나 더
            DubeolSikState::ChoseongJungSeong(ch) => {
                let (cho, _jung, _) = decompose_syllable(ch);

                *self = DubeolSikState::Choseong(cho);
                InputResult::Preedit(cho_to_char(cho))
            }
            // 강
            DubeolSikState::Complete(ch) => {
                let (cho, jung, jong) = decompose_syllable(ch);

                let mut ch = compose_syllable(cho, jung).unwrap();
                let (jong_left, _jong_right) = try_decompose_jong(jong);

                // '없' -> '업'

                if jong_left != '\0' {
                    ch = compose_syllable(ch, jong_left).unwrap();
                    *self = DubeolSikState::Complete(ch);
                } else {
                    *self = DubeolSikState::ChoseongJungSeong(ch);
                }

                InputResult::Preedit(ch)
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

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn test_input(inputs: &[(u8, InputResult)]) {
        let mut layout = DubeolSik::new();

        for (code, expect_result) in inputs.iter().copied() {
            assert_eq!(expect_result, layout.map_key(code));
        }
    }

    #[test]
    fn jo_to_cho() {
        assert_eq!(jong_direct_to_cho('ᆺ'), 'ᄉ');
        assert_eq!(jong_to_cho('ᆭ'), ('ᄂ', 'ᄒ'));
    }

    #[test]
    fn decompose_jong() {
        assert_eq!(try_decompose_jong('ᆭ'), ('ᆫ', 'ᇂ'));
        assert_eq!(try_decompose_jong('ᆹ'), ('ᆸ', 'ᆺ'));
    }

    #[test]
    fn com_moum() {
        test_input(&[
            (D, InputResult::Preedit('ㅇ')),
            (H, InputResult::Preedit('오')),
            (L, InputResult::Preedit('외')),
            (D, InputResult::Preedit('욍')),
            (D, InputResult::CommitPreedit('욍', 'ㅇ')),
            (K, InputResult::Preedit('아')),
            (S, InputResult::Preedit('안')),
            (G, InputResult::Preedit('않')),
            (E, InputResult::CommitPreedit('않', 'ㄷ')),
        ]);
    }

    #[test]
    fn backspace() {
        test_input(&[
            (R, InputResult::Preedit('ㄱ')),
            (K, InputResult::Preedit('가')),
            (D, InputResult::Preedit('강')),
            (BS, InputResult::Preedit('가')),
            (BS, InputResult::Preedit('ㄱ')),
            (BS, InputResult::ClearPreedit),
            (R, InputResult::Preedit('ㄱ')),
        ])
    }

    #[test]
    fn compose_jong() {
        test_input(&[
            (D, InputResult::Preedit('ㅇ')),
            (J, InputResult::Preedit('어')),
            (Q, InputResult::Preedit('업')),
            (T, InputResult::Preedit('없')),
            (BS, InputResult::Preedit('업')),
        ])
    }
}
