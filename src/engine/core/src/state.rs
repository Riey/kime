use crate::{
    characters::{Choseong, JongToCho, Jongseong, Jungseong},
    Config, InputResult,
};

/// 한글 입력 오토마타
#[derive(Debug, Clone)]
pub struct HangulState {
    state: CharacterState,
    word_commit: bool,
    characters: String,
    buf: String,
}

impl HangulState {
    pub fn new(word_commit: bool) -> Self {
        Self {
            state: CharacterState::new(),
            word_commit,
            characters: String::with_capacity(64),
            buf: String::with_capacity(64),
        }
    }

    pub fn commit_str(&mut self) -> &str {
        if self.word_commit {
            self.buf.clear();
            self.buf.push_str(&self.characters);
            self.state.write(&mut self.buf);
            &self.buf
        } else {
            &self.characters
        }
    }

    pub fn preedit_str(&mut self) -> &str {
        if self.word_commit {
            self.commit_str()
        } else {
            self.buf.clear();
            self.state.write(&mut self.buf);
            &self.buf
        }
    }

    pub fn pass(&mut self, s: &str) {
        self.state.write(&mut self.characters);
        self.characters.push_str(s);
        self.state.reset();
    }

    pub fn clear_preedit(&mut self) {
        self.state.write(&mut self.characters);
        self.state.reset();
    }

    pub fn flush(&mut self) {
        self.characters.clear();
    }

    pub fn reset(&mut self) {
        self.state.reset();
        self.characters.clear();
    }

    pub fn preedit_result(&self) -> InputResult {
        if self.state.need_display() || self.word_commit && !self.characters.is_empty() {
            InputResult::HAS_PREEDIT
        } else {
            InputResult::empty()
        }
    }

    fn convert_result(&mut self, ret: CharacterResult) -> InputResult {
        match ret {
            CharacterResult::Consume => self.preedit_result() | InputResult::CONSUMED,
            CharacterResult::NewCharacter(new) => {
                debug_assert!(self.state.need_display());

                self.characters.push(self.state.to_char());
                self.state = new;

                if self.word_commit {
                    InputResult::HAS_PREEDIT | InputResult::CONSUMED
                } else {
                    InputResult::NEED_FLUSH | self.preedit_result() | InputResult::CONSUMED
                }
            }
        }
    }

    pub fn backspace(&mut self, config: &Config) -> InputResult {
        loop {
            if self.state.backspace(config) {
                return self.preedit_result() | InputResult::CONSUMED;
            }

            match self.characters.pop().map(CharacterState::load_from_ch) {
                Some(new_last) => {
                    self.state = new_last;
                }
                None => {
                    return InputResult::empty();
                }
            }
        }
    }

    pub fn cho_jong(
        &mut self,
        cho: Choseong,
        jong: Jongseong,
        first: bool,
        config: &Config,
    ) -> InputResult {
        let ret = self.state.cho_jong(cho, jong, first, config);
        self.convert_result(ret)
    }

    pub fn cho_jung(
        &mut self,
        cho: Choseong,
        jung: Jungseong,
        first: bool,
        config: &Config,
    ) -> InputResult {
        let ret = self.state.cho_jung(cho, jung, first, config);
        self.convert_result(ret)
    }

    pub fn jung_jong(
        &mut self,
        jung: Jungseong,
        jong: Jongseong,
        first: bool,
        config: &Config,
    ) -> InputResult {
        let ret = self.state.jung_jong(jung, jong, first, config);
        self.convert_result(ret)
    }

    pub fn cho(&mut self, cho: Choseong, config: &Config) -> InputResult {
        let ret = self.state.cho(cho, config);
        self.convert_result(ret)
    }

    pub fn jung(&mut self, jung: Jungseong, config: &Config) -> InputResult {
        let ret = self.state.jung(jung, config);
        self.convert_result(ret)
    }

    pub fn jong(&mut self, jong: Jongseong, config: &Config) -> InputResult {
        let ret = self.state.jong(jong, config);
        self.convert_result(ret)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CharacterResult {
    Consume,
    NewCharacter(CharacterState),
}

/// 한글 글자 상태
#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
struct CharacterState {
    cho: Option<Choseong>,
    jung: Option<Jungseong>,
    jong: Option<Jongseong>,
}

impl CharacterState {
    pub const fn new() -> Self {
        Self {
            cho: None,
            jung: None,
            jong: None,
        }
    }

    pub fn load_from_ch(ch: char) -> Self {
        match Choseong::decompose(ch) {
            Some((cho, jung, jong)) => Self {
                cho: Some(cho),
                jung: Some(jung),
                jong,
            },
            _ => Self::new(),
        }
    }

    pub fn reset(&mut self) {
        self.cho = None;
        self.jung = None;
        self.jong = None;
    }

    pub fn to_char(&self) -> char {
        match (self.cho, self.jung, self.jong) {
            (None, None, None) |
            // can't be char
            (None, Some(_), Some(_)) |
            (Some(_), None, Some(_)) => '\0',

            (Some(cho), Some(jung), jong) => cho.compose(jung, jong),

            (Some(cho), None, None) => cho.jamo(),
            (None, Some(jung), None) => jung.jamo(),
            (None, None, Some(jong)) => jong.jamo(),
        }
    }

    pub fn write(&self, out: &mut String) {
        let ch = self.to_char();

        if ch != '\0' {
            out.push(ch);
        }
    }

    pub const fn need_display(&self) -> bool {
        match (self.cho, self.jung, self.jong) {
            (None, None, None) |
            // can't be char
            (None, Some(_), Some(_)) |
            (Some(_), None, Some(_)) => false,
            _ => true,
        }
    }

    pub fn backspace(&mut self, config: &Config) -> bool {
        if let Some(jong) = self.jong.as_mut() {
            if let Some(new_jong) = jong.backspace(config) {
                *jong = new_jong;
            } else {
                self.jong = None;
            }
        } else if let Some(jung) = self.jung.as_mut() {
            if let Some(new_jung) = jung.backspace(config) {
                *jung = new_jung;
            } else {
                self.jung = None;
            }
        } else if let Some(cho) = self.cho.as_mut() {
            if let Some(new_cho) = cho.backspace(config) {
                *cho = new_cho;
            } else {
                self.cho = None;
            }
        } else {
            // empty state
            return false;
        }

        true
    }

    pub fn cho_jong(
        &mut self,
        cho: Choseong,
        jong: Jongseong,
        first: bool,
        config: &Config,
    ) -> CharacterResult {
        if self.cho.is_none() || self.jung.is_none() {
            self.cho(cho, config)
        } else if self.jung.is_some() {
            self.jong(jong, config)
        } else if first {
            self.cho(cho, config)
        } else {
            self.jong(jong, config)
        }
    }

    pub fn cho_jung(
        &mut self,
        cho: Choseong,
        jung: Jungseong,
        first: bool,
        config: &Config,
    ) -> CharacterResult {
        if self.cho.is_none() || self.jung.is_some() {
            self.cho(cho, config)
        } else if self.cho.is_some() {
            self.jung(jung, config)
        } else if first {
            self.cho(cho, config)
        } else {
            self.jung(jung, config)
        }
    }

    pub fn jung_jong(
        &mut self,
        jung: Jungseong,
        jong: Jongseong,
        first: bool,
        config: &Config,
    ) -> CharacterResult {
        // 아 + $ㄴㅖ = 안
        if self.jung.is_some() {
            self.jong(jong, config)
        } else if self.cho.is_some() {
            self.jung(jung, config)
        } else if first {
            self.jong(jong, config)
        } else {
            self.jung(jung, config)
        }
    }

    pub fn cho(&mut self, cho: Choseong, config: &Config) -> CharacterResult {
        if let Some(prev_cho) = self.cho {
            if self.jong.is_some() {
                CharacterResult::NewCharacter(Self {
                    cho: Some(cho),
                    ..Default::default()
                })
            } else {
                match prev_cho.try_add(cho, config) {
                    Some(new) => {
                        self.cho = Some(new);
                        CharacterResult::Consume
                    }
                    None => CharacterResult::NewCharacter(Self {
                        cho: Some(cho),
                        ..Default::default()
                    }),
                }
            }
        } else {
            self.cho = Some(cho);
            CharacterResult::Consume
        }
    }

    pub fn jung(&mut self, jung: Jungseong, config: &Config) -> CharacterResult {
        if let Some(jong) = self.jong {
            if self.cho.is_some() {
                // has choseong move jongseong to next choseong
                let new;

                match jong.to_cho(config) {
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

                return CharacterResult::NewCharacter(new);
            } else {
                // only jongseong commit replace with jungseong
                return CharacterResult::NewCharacter(Self {
                    cho: None,
                    jung: Some(jung),
                    jong: None,
                });
            }
        }

        if let Some(prev_jung) = self.jung {
            match prev_jung.try_add(jung, config) {
                Some(new) => {
                    self.jung = Some(new);
                    CharacterResult::Consume
                }
                None => {
                    let new;

                    if let Some(jong) = self.jong {
                        match jong.to_cho(config) {
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

                    CharacterResult::NewCharacter(new)
                }
            }
        } else {
            self.jung = Some(jung);
            CharacterResult::Consume
        }
    }

    pub fn jong(&mut self, jong: Jongseong, config: &Config) -> CharacterResult {
        if let Some(prev_jong) = self.jong {
            match prev_jong.try_add(jong, config) {
                Some(new) => {
                    self.jong = Some(new);
                    CharacterResult::Consume
                }
                None => {
                    let new;

                    match jong.to_cho(config) {
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

                    CharacterResult::NewCharacter(new)
                }
            }
        } else {
            self.jong = Some(jong);
            CharacterResult::Consume
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

        state.cho_jong(Choseong::Ieung, Jongseong::Ieung, true, &config);
        state.jung(Jungseong::A, &config);
        state.cho_jong(Choseong::Ieung, Jongseong::Ieung, true, &config);

        assert_eq!(
            CharacterResult::NewCharacter(CharacterState {
                cho: Some(Choseong::Ieung),
                jung: Some(Jungseong::A),
                jong: None
            }),
            state.jung(Jungseong::A, &config)
        );
    }
}
