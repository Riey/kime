use super::characters::{Choseong, JongToCho, Jongseong, Jungseong};
use super::InputResult;

/// 한글 입력 오토마타
#[derive(Debug, Default, Clone, Copy)]
pub struct CharacterState {
    cho: Option<Choseong>,
    jung: Option<Jungseong>,
    jong: Option<Jongseong>,
}

impl CharacterState {
    pub fn to_char(&self) -> char {
        match (self.cho, self.jung, self.jong) {
            (None, None, None) => '\0',
            (Some(cho), Some(jung), jong) => cho.compose(jung, jong),

            (Some(cho), None, None) => cho.jamo(),
            (None, Some(jung), None) => jung.jamo(),
            (None, None, Some(jong)) => jong.jamo(),

            // can't be render just workaround
            (None, Some(_jung), Some(jong)) => jong.jamo(),
            (Some(cho), None, Some(_jong)) => cho.jamo(),
        }
    }

    pub fn reset(&mut self) -> Option<char> {
        let pe = self.preedit_char();
        self.cho = None;
        self.jung = None;
        self.jong = None;
        pe
    }

    pub fn preedit_char(&self) -> Option<char> {
        match (self.cho, self.jung, self.jong) {
            (None, None, None) => None,
            (Some(cho), Some(jung), jong) => Some(cho.compose(jung, jong)),

            (Some(cho), None, None) => Some(cho.jamo()),
            (None, Some(jung), None) => Some(jung.jamo()),
            (None, None, Some(jong)) => Some(jong.jamo()),

            // can't be char
            (None, Some(_jung), Some(_jong)) => None,
            (Some(_cho), None, Some(_jong)) => None,
        }
    }

    /// Replace self with new then return previous status char
    fn replace(&mut self, new: Self) -> char {
        let prev = std::mem::replace(self, new);
        prev.to_char()
    }

    pub fn backspace(&mut self) -> InputResult {
        if let Some(jong) = self.jong.as_mut() {
            if let Some(new_jong) = jong.backspace() {
                *jong = new_jong;
            } else {
                self.jong = None;
            }
        } else if let Some(jung) = self.jung.as_mut() {
            if let Some(new_jung) = jung.backspace() {
                *jung = new_jung;
            } else {
                self.jung = None;
            }
        } else if let Some(cho) = self.cho.as_mut() {
            if let Some(new_cho) = cho.backspace() {
                *cho = new_cho;
            } else {
                self.cho = None;
            }
        } else {
            // empty
            return InputResult::Bypass;
        }

        let ch = self.to_char();

        if ch == '\0' {
            InputResult::ClearPreedit
        } else {
            InputResult::Preedit(ch)
        }
    }

    // 두벌식용
    pub fn cho_jong(&mut self, cho: Choseong, jong: Jongseong) -> InputResult {
        if self.jung.is_none() {
            self.cho(cho)
        } else {
            self.jong(jong)
        }
    }

    pub fn cho(&mut self, cho: Choseong) -> InputResult {
        if let Some(prev_cho) = self.cho {
            match prev_cho.try_add(cho) {
                Some(new) => {
                    self.cho = Some(new);
                    InputResult::Preedit(self.to_char())
                }
                None => {
                    let commit = self.replace(Self {
                        cho: Some(cho),
                        ..Default::default()
                    });
                    InputResult::CommitPreedit(commit, self.to_char())
                }
            }
        } else {
            self.cho = Some(cho);
            InputResult::Preedit(self.to_char())
        }
    }

    pub fn jung(&mut self, jung: Jungseong) -> InputResult {
        // TODO: try decompose jongseong
        if let Some(prev_jung) = self.jung {
            match prev_jung.try_add(jung) {
                Some(new) => {
                    self.jung = Some(new);
                    InputResult::Preedit(self.to_char())
                }
                None => {
                    let new;

                    if let Some(jong) = self.jong {
                        match jong.to_cho() {
                            JongToCho::Direct(cho) => {
                                self.jong = None;
                                new = Self {
                                    cho: Some(cho),
                                    jung: Some(jung),
                                    ..Default::default()
                                };
                            }
                            JongToCho::Compose(left, cho) => {
                                self.jong = Some(left);
                                new = Self {
                                    cho: Some(cho),
                                    jung: Some(jung),
                                    ..Default::default()
                                };
                            }
                        }
                    } else {
                        new = Self {
                            jung: Some(jung),
                            ..Default::default()
                        };
                    }

                    let commit = self.replace(new);
                    InputResult::CommitPreedit(commit, self.to_char())
                }
            }
        } else {
            self.jung = Some(jung);
            InputResult::Preedit(self.to_char())
        }
    }

    pub fn jong(&mut self, jong: Jongseong) -> InputResult {
        if let Some(prev_jong) = self.jong {
            match prev_jong.try_add(jong) {
                Some(new) => {
                    self.jong = Some(new);
                    InputResult::Preedit(self.to_char())
                }
                None => {
                    let new;

                    match prev_jong.to_cho() {
                        JongToCho::Direct(cho) => {
                            new = Self {
                                cho: Some(cho),
                                ..Default::default()
                            };
                        }
                        JongToCho::Compose(..) => {
                            new = Self {
                                jong: Some(jong),
                                ..Default::default()
                            };
                        }
                    }

                    let commit = self.replace(new);
                    InputResult::CommitPreedit(commit, self.to_char())
                }
            }
        } else {
            self.jong = Some(jong);
            InputResult::Preedit(self.to_char())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jong() {
        let mut state = CharacterState::default();

        assert_eq!(
            InputResult::Preedit('ㅇ'),
            state.cho_jong(Choseong::Ieung, Jongseong::Ieung)
        );
        assert_eq!(InputResult::Preedit('아'), state.jung(Jungseong::A));
        assert_eq!(
            InputResult::Preedit('앙'),
            state.cho_jong(Choseong::Ieung, Jongseong::Ieung)
        );
        assert_eq!(
            InputResult::CommitPreedit('아', '아'),
            state.jung(Jungseong::A)
        );
    }
}
