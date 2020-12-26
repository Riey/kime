use super::{
    compose::{compose_syllable, decompose_syllable, jamo_to_chara, jong_to_cho},
    keycode::*,
    InputLayout, InputResult,
};

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
                InputResult::Commit(jamo_to_chara(ch))
            }
            DubeolSikState::JungSeong(ch) => {
                let ch = compose_syllable(choseong, ch).unwrap();
                *self = DubeolSikState::ChoseongJungSeong(ch);
                InputResult::Preedit(jamo_to_chara(ch))
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
                InputResult::Commit(jamo_to_chara(ch))
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
            DubeolSikState::Choseong(ch)
            | DubeolSikState::ChoseongJungSeong(ch)
            | DubeolSikState::JungSeong(ch)
            | DubeolSikState::Complete(ch) => {
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

impl InputLayout for DubeolSik {
    fn map_key(&mut self, keycode: u8) -> InputResult {
        match keycode {
            Q => self.state.jaum('ᄇ', 'ᆸ'),
            W => self.state.jaum('ᄌ', 'ᆽ'),
            E => self.state.jaum('ᄃ', 'ᆮ'),
            R => self.state.jaum('ᄀ', 'ᆨ'),
            T => self.state.jaum('ᄉ', 'ᆺ'),
            Y => self.state.moum('ᅭ'),
            I => self.state.moum('ᅣ'),
            O => self.state.moum('ᅢ'),
            P => self.state.moum('ᅦ'),
            A => self.state.jaum('ᄆ', 'ᆷ'),
            S => self.state.jaum('ᄂ', 'ᆫ'),
            D => self.state.jaum('ᄋ', 'ᆼ'),
            F => self.state.jaum('ᄅ', 'ᆯ'),
            G => self.state.jaum('ᄒ', 'ᇂ'),
            H => self.state.moum('ᅩ'),
            J => self.state.moum('ᅥ'),
            K => self.state.moum('ᅡ'),
            L => self.state.moum('ᅵ'),

            Z => self.state.jaum('ᄏ', 'ᆿ'),
            X => self.state.jaum('ᄐ', 'ᇀ'),
            C => self.state.jaum('ᄎ', 'ᆾ'),
            V => self.state.jaum('ᄑ', 'ᇁ'),
            B => self.state.moum('ᅲ'),
            N => self.state.moum('ᅮ'),
            M => self.state.moum('ᅳ'),
            _ => self.state.other(),
        }
    }
}
