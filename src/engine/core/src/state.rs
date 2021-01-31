use super::characters::{Choseong, JongToCho, Jongseong, Jungseong};
use super::InputResult;
use crate::Config;

/// 한글 입력 오토마타
#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct CharacterState {
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

    pub fn reset(&mut self) -> char {
        let cc = self.commit_char();
        self.cho = None;
        self.jung = None;
        self.jong = None;
        cc
    }

    pub fn commit_char(&self) -> char {
        match (self.cho, self.jung, self.jong) {
            (None, None, None) => '\0',
            (Some(cho), Some(jung), jong) => cho.compose(jung, jong),

            (Some(cho), None, None) => cho.jamo(),
            (None, Some(jung), None) => jung.jamo(),
            (None, None, Some(jong)) => jong.jamo(),

            // can't be char
            (None, Some(_jung), Some(_jong)) => '\0',
            (Some(_cho), None, Some(_jong)) => '\0',
        }
    }

    /// Replace self with new then return previous status char
    fn replace(&mut self, new: Self) -> InputResult {
        let prev = std::mem::replace(self, new);

        match prev.commit_char() {
            '\0' => InputResult::preedit(self.to_char()),
            prev => InputResult::commit_preedit(prev, self.to_char()),
        }
    }

    pub fn backspace(&mut self, config: &Config) -> InputResult {
        if let Some(jong) = self.jong.as_mut() {
            if let Some(new_jong) = jong.backspace(&config.compose) {
                *jong = new_jong;
            } else {
                self.jong = None;
            }
        } else if let Some(jung) = self.jung.as_mut() {
            if let Some(new_jung) = jung.backspace(&config.compose) {
                *jung = new_jung;
            } else {
                self.jung = None;
            }
        } else if let Some(cho) = self.cho.as_mut() {
            if let Some(new_cho) = cho.backspace(&config.compose) {
                *cho = new_cho;
            } else {
                self.cho = None;
            }
        } else {
            // empty
            return InputResult::bypass();
        }

        let ch = self.to_char();

        if ch == '\0' {
            InputResult::clear_preedit()
        } else {
            InputResult::preedit(ch)
        }
    }

    // 두벌식용
    pub fn cho_jong(&mut self, cho: Choseong, jong: Jongseong, config: &Config) -> InputResult {
        if self.cho.is_none() || self.jung.is_none() {
            self.cho(cho, config)
        } else {
            self.jong(jong, config)
        }
    }

    pub fn cho(&mut self, cho: Choseong, config: &Config) -> InputResult {
        if let Some(prev_cho) = self.cho {
            match prev_cho.try_add(cho, &config.compose) {
                Some(new) => {
                    self.cho = Some(new);
                    InputResult::preedit(self.to_char())
                }
                None => self.replace(Self {
                    cho: Some(cho),
                    ..Default::default()
                }),
            }
        } else {
            self.cho = Some(cho);
            InputResult::preedit(self.to_char())
        }
    }

    pub fn jung(&mut self, jung: Jungseong, config: &Config) -> InputResult {
        if let Some(jong) = self.jong {
            if self.cho.is_some() {
                // has choseong move jongseong to next choseong
                let new;

                match jong.to_cho(&config.compose) {
                    JongToCho::Direct(cho) => {
                        self.jong = None;
                        new = Self {
                            cho: Some(cho),
                            jung: Some(jung),
                            jong: None,
                        };
                    }
                    JongToCho::Compose(jong, cho) => {
                        self.jong = Some(jong);
                        new = Self {
                            cho: Some(cho),
                            jung: Some(jung),
                            jong: None,
                        };
                    }
                }

                return self.replace(new);
            } else {
                // only jongseong commit replace with jungseong
                return self.replace(Self {
                    cho: None,
                    jung: Some(jung),
                    jong: None,
                });
            }
        }

        if let Some(prev_jung) = self.jung {
            match prev_jung.try_add(jung, &config.compose) {
                Some(new) => {
                    self.jung = Some(new);
                    InputResult::preedit(self.to_char())
                }
                None => {
                    let new;

                    if let Some(jong) = self.jong {
                        match jong.to_cho(&config.compose) {
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

                    self.replace(new)
                }
            }
        } else {
            self.jung = Some(jung);
            InputResult::preedit(self.to_char())
        }
    }

    pub fn jong(&mut self, jong: Jongseong, config: &Config) -> InputResult {
        if let Some(prev_jong) = self.jong {
            match prev_jong.try_add(jong, &config.compose) {
                Some(new) => {
                    self.jong = Some(new);
                    InputResult::preedit(self.to_char())
                }
                None => {
                    let new;

                    match jong.to_cho(&config.compose) {
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

                    self.replace(new)
                }
            }
        } else {
            self.jong = Some(jong);
            InputResult::preedit(self.to_char())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jong() {
        let mut state = CharacterState::default();
        let config = Config::default();

        assert_eq!(
            InputResult::preedit('ㅇ'),
            state.cho_jong(Choseong::Ieung, Jongseong::Ieung, &config)
        );
        assert_eq!(
            InputResult::preedit('아'),
            state.jung(Jungseong::A, &config)
        );
        assert_eq!(
            InputResult::preedit('앙'),
            state.cho_jong(Choseong::Ieung, Jongseong::Ieung, &config)
        );
        assert_eq!(
            InputResult::commit_preedit('아', '아'),
            state.jung(Jungseong::A, &config)
        );
    }
}
