use super::characters::{Choseong, Jongseong, Jungseong};
use super::InputResult;
use std::mem;

#[derive(Debug, Clone, Copy)]
pub enum CharacterState {
    Empty,
    Cho(Choseong),
    Jung(Jungseong),
    ChoJung(Choseong, Jungseong),
    ChoJungJong(Choseong, Jungseong, Jongseong),
}

impl Default for CharacterState {
    fn default() -> Self {
        CharacterState::Empty
    }
}

impl CharacterState {
    fn to_char(&self) -> char {
        match self {
            CharacterState::Empty => '\0',
            CharacterState::Cho(cho) => cho.jamo(),
            CharacterState::Jung(jung) => jung.jamo(),
            CharacterState::ChoJung(cho, jung) => cho.compose(*jung, None),
            CharacterState::ChoJungJong(cho, jung, jong) => cho.compose(*jung, Some(*jong)),
        }
    }

    pub fn cho_jong(&mut self, cho: Choseong, jong: Jongseong) -> InputResult {
        if matches!(
            self,
            CharacterState::ChoJung(..) | CharacterState::ChoJungJong(..)
        ) {
            self.jong(jong)
        } else {
            self.cho(cho)
        }
    }

    pub fn cho(&mut self, cho: Choseong) -> InputResult {
        match *self {
            CharacterState::Empty => {
                *self = CharacterState::Cho(cho);
                InputResult::Preedit(cho.jamo())
            }
            CharacterState::Cho(prev_cho) => match prev_cho {
                Choseong::Giyeok => {
                    *self = CharacterState::Cho(Choseong::SsangGiyeok);
                    InputResult::Preedit(Choseong::Giyeok.jamo())
                }
                _ => {
                    *self = CharacterState::Cho(cho);
                    InputResult::CommitPreedit(prev_cho.jamo(), cho.jamo())
                }
            },
            _ => todo!(),
        }
    }

    pub fn jung(&mut self, jung: Jungseong) -> InputResult {
        match *self {
            CharacterState::Empty => {
                *self = CharacterState::Jung(jung);
                InputResult::Preedit(jung.jamo())
            }
            CharacterState::Cho(cho) => {
                *self = CharacterState::ChoJung(cho, jung);
                InputResult::Preedit(cho.compose(jung, None))
            }
            CharacterState::Jung(..) | CharacterState::ChoJung(..) => todo!("compose jungseong"),
            CharacterState::ChoJungJong(cho, prev_jung, jong) => {
                todo!("split jongseong")
            }
        }
    }

    pub fn jong(&mut self, jong: Jongseong) -> InputResult {
        todo!()
    }
}
