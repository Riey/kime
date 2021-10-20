use enumset::EnumSet;

use crate::{
    characters::{Choseong, JongToCho, Jongseong, Jungseong, KeyValue},
    Addon,
};

/// 한글 입력 오토마타
#[derive(Debug, Clone)]
pub struct HangulEngine {
    state: CharacterState,
    word_commit: bool,
    preedit_filler: bool,
    word_buf: String,
}

impl HangulEngine {
    pub fn new(word_commit: bool, preedit_filler: bool) -> Self {
        Self {
            state: CharacterState::new(),
            word_commit,
            preedit_filler,
            word_buf: String::new(),
        }
    }

    pub fn has_preedit(&self) -> bool {
        self.state.need_display() || !self.word_buf.is_empty()
    }

    pub fn preedit_str(&self, buf: &mut String) {
        buf.push_str(&self.word_buf);
        self.state.display(self.preedit_johab, buf);
    }

    pub fn clear_preedit(&mut self, commit_buf: &mut String) {
        commit_buf.push_str(&self.word_buf);
        self.word_buf.clear();
        self.state.to_str(commit_buf);
        self.state.reset();
    }

    pub fn reset(&mut self) {
        self.word_buf.clear();
        self.state.reset();
    }

    fn convert_result(&mut self, ret: CharacterResult, commit_buf: &mut String) -> bool {
        match ret {
            CharacterResult::Consume => true,
            CharacterResult::NewCharacter(new_ch) => {
                    if self.word_commit {
                    self.state.to_str(&mut self.word_buf);
                    } else {
                    self.state.to_str(commit_buf);
                }
                self.state = new_ch;
                true
            }
        }
    }

    pub fn backspace(&mut self, addons: EnumSet<Addon>, commit_buf: &mut String) -> bool {
        if self.state.backspace(addons) {
            true
        } else if commit_buf.pop().is_some() {
            true
        } else {
            false
        }
    }

    pub fn key(&mut self, kv: KeyValue, addons: EnumSet<Addon>, commit_buf: &mut String) -> bool {
        let ret = match kv {
            KeyValue::Pass(pass) => {
                self.clear_preedit(commit_buf);
                commit_buf.push(pass);
                return true;
            }
            KeyValue::Choseong { cho } => self.state.cho(cho, addons),
            KeyValue::Jungseong { jung, compose } => self.state.jung(jung, compose, addons),
            KeyValue::Jongseong { jong } => self.state.jong(jong, addons),
            KeyValue::ChoJong { cho, jong, first } => self.state.cho_jong(cho, jong, first, addons),
            KeyValue::ChoJung {
                cho,
                jung,
                first,
                compose,
            } => self.state.cho_jung(cho, jung, first, compose, addons),
            KeyValue::JungJong {
                jung,
                jong,
                first,
                compose,
            } => self.state.jung_jong(jung, jong, first, compose, addons),
        };

        self.convert_result(ret, commit_buf)
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
    /// 조합용 중성
    compose_jung: bool,
    jong: Option<Jongseong>,
}

impl CharacterState {
    pub const fn new() -> Self {
        Self {
            cho: None,
            jung: None,
            compose_jung: false,
            jong: None,
        }
    }

    pub fn reset(&mut self) {
        self.cho = None;
        self.jung = None;
        self.jong = None;
    }

    pub fn display(&self, preedit_johab: bool, out: &mut String) {
        if preedit_johab {
            self.johab_str(out);
        } else {
            self.to_str(out);
        }
    }

    pub fn johab_str(&self, out: &mut String) {
        match (self.cho, self.jung, self.jong) {
            (None, None, None) => {}
            (None, Some(jung), Some(jong)) => {
                out.push(Choseong::FILLER);
                out.push(jung.into());
                out.push(jong.into());
            }
            (Some(cho), None, Some(jong)) => {
                out.push(cho.into());
                out.push(Jungseong::FILLER);
                out.push(jong.into());
            }

            (Some(cho), Some(jung), jong) => out.push(cho.compose(jung, jong)),

            (Some(cho), None, None) => out.push(cho.jamo()),
            (None, Some(jung), None) => out.push(jung.jamo()),
            (None, None, Some(jong)) => out.push(jong.jamo()),
        }
    }

    pub fn to_str(&self, out: &mut String) {
        match (self.cho, self.jung, self.jong) {
            (None, None, None) => {}
            (None, Some(jung), Some(jong)) => {
                out.push(jung.jamo());
                out.push(jong.jamo());
            }
            (Some(cho), None, Some(jong)) => {
                out.push(cho.jamo());
                out.push(jong.jamo());
            }
            (Some(cho), Some(jung), jong) => out.push(cho.compose(jung, jong)),
            (Some(cho), None, None) => out.push(cho.jamo()),
            (None, Some(jung), None) => out.push(jung.jamo()),
            (None, None, Some(jong)) => out.push(jong.jamo()),
        }
    }

    pub const fn need_display(&self) -> bool {
        match (self.cho, self.jung, self.jong) {
            (None, None, None) => false,
            _ => true,
        }
    }

    pub fn backspace(&mut self, addons: EnumSet<Addon>) -> bool {
        if let Some(jong) = self.jong.as_mut() {
            if let Some(new_jong) = jong.backspace(addons) {
                *jong = new_jong;
            } else {
                self.jong = None;
            }
        } else if let Some(jung) = self.jung.as_mut() {
            if let Some(new_jung) = jung.backspace(addons) {
                *jung = new_jung;
                self.compose_jung = true;
            } else {
                self.jung = None;
                self.compose_jung = false;
            }
        } else if let Some(cho) = self.cho.as_mut() {
            if let Some(new_cho) = cho.backspace(addons) {
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

    fn choseong_can_compose_jongseong(&self, cho: Choseong, addons: EnumSet<Addon>) -> bool {
        self.jong.map_or(false, |j| match j.to_cho(addons) {
            JongToCho::Direct(prev_cho) | JongToCho::Compose(_, prev_cho) => {
                prev_cho.try_add(cho, addons).is_some()
            }
        })
    }

    // 갈마들이 입력

    pub fn cho_jong(
        &mut self,
        cho: Choseong,
        jong: Jongseong,
        first: bool,
        addons: EnumSet<Addon>,
    ) -> CharacterResult {
        if self.cho.is_none()
            || self.jung.is_none()
            || addons.contains(Addon::TreatJongseongAsChoseongCompose)
                && self.choseong_can_compose_jongseong(cho, addons)
        {
            self.cho(cho, addons)
        } else if self.jung.is_some() || !first {
            self.jong(jong, addons)
        } else {
            self.cho(cho, addons)
        }
    }

    pub fn cho_jung(
        &mut self,
        cho: Choseong,
        jung: Jungseong,
        first: bool,
        compose_jung: bool,
        addons: EnumSet<Addon>,
    ) -> CharacterResult {
        if self.cho.is_some()
            && self.jung.map_or(true, |j| {
                self.compose_jung && j.try_add(jung, addons).is_some()
            })
        {
            self.jung(jung, compose_jung, addons)
        } else if self.cho.is_none() || first {
            self.cho(cho, addons)
        } else {
            self.jung(jung, compose_jung, addons)
        }
    }

    pub fn jung_jong(
        &mut self,
        jung: Jungseong,
        jong: Jongseong,
        first: bool,
        compose_jung: bool,
        addons: EnumSet<Addon>,
    ) -> CharacterResult {
        // 아 + $ㄴㅖ = 안
        // ㅇ + $ㅜ + $ㅊㅔ = 웨
        // ㅇ + ㅜ + $ㅊㅔ = 웇
        if self.jung.map_or(true, |j| {
            self.compose_jung && j.try_add(jung, addons).is_some()
        }) {
            self.jung(jung, compose_jung, addons)
        } else if self.cho.is_some() || !first {
            self.jong(jong, addons)
        } else {
            self.jung(jung, compose_jung, addons)
        }
    }

    // 일반 입력

    pub fn cho(&mut self, mut cho: Choseong, addons: EnumSet<Addon>) -> CharacterResult {
        if let Some(prev_cho) = self.cho {
            if let Some(jong) = self.jong {
                if addons.contains(Addon::TreatJongseongAsChoseongCompose) {
                    match jong.to_cho(addons) {
                        JongToCho::Direct(prev_cho) => {
                            if let Some(new_cho) = prev_cho.try_add(cho, addons) {
                                self.jong = None;
                                cho = new_cho;
                            }
                        }
                        JongToCho::Compose(jong, prev_cho) => {
                            if let Some(new_cho) = prev_cho.try_add(cho, addons) {
                                self.jong = Some(jong);
                                cho = new_cho;
                            }
                        }
                    }
                }

                CharacterResult::NewCharacter(Self {
                    cho: Some(cho),
                    ..Default::default()
                })
            } else {
                match prev_cho.try_add(cho, addons) {
                    Some(new) if self.jung.is_none() => {
                        self.cho = Some(new);
                        CharacterResult::Consume
                    }
                    _ => CharacterResult::NewCharacter(Self {
                        cho: Some(cho),
                        ..Default::default()
                    }),
                }
            }
        } else if addons.contains(Addon::FlexibleComposeOrder)
            || self.jung.is_none() && self.jong.is_none()
        {
            self.cho = Some(cho);
            CharacterResult::Consume
        } else {
            CharacterResult::NewCharacter(Self {
                cho: Some(cho),
                ..Default::default()
            })
        }
    }

    pub fn jung(
        &mut self,
        jung: Jungseong,
        compose_jung: bool,
        addons: EnumSet<Addon>,
    ) -> CharacterResult {
        if addons.contains(Addon::TreatJongseongAsChoseong) {
            if let Some(jong) = self.jong {
                if self.cho.is_some() {
                    // has choseong move jongseong to next choseong
                    let new;

                    match jong.to_cho(addons) {
                        JongToCho::Direct(cho) => {
                            self.jong = None;
                            new = Self {
                                cho: Some(cho),
                                jung: Some(jung),
                                jong: None,
                                compose_jung,
                            };
                        }
                        JongToCho::Compose(jong, cho) => {
                            self.jong = Some(jong);
                            new = Self {
                                cho: Some(cho),
                                jung: Some(jung),
                                jong: None,
                                compose_jung,
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
                        compose_jung,
                    });
                }
            }
        }

        if let Some(prev_jung) = self.jung {
            match prev_jung.try_add(jung, addons) {
                Some(new) if self.compose_jung => {
                    self.jung = Some(new);
                    self.compose_jung = false;
                    CharacterResult::Consume
                }
                _ => CharacterResult::NewCharacter(Self {
                    jung: Some(jung),
                    compose_jung,
                    ..Default::default()
                }),
            }
        } else if let Some(_) = self.jong {
            // $ㅁ + ㅏ = ㅁㅏ
            // 초성없이 중성과 종성만 있는 경우를 배제
            CharacterResult::NewCharacter(Self {
                jung: Some(jung),
                compose_jung,
                ..Default::default()
            })
        } else {
            self.jung = Some(jung);
            self.compose_jung = compose_jung;
            CharacterResult::Consume
        }
    }

    pub fn jong(&mut self, jong: Jongseong, addons: EnumSet<Addon>) -> CharacterResult {
        if let Some(prev_jong) = self.jong {
            match prev_jong.try_add(jong, addons) {
                Some(new) => {
                    self.jong = Some(new);
                    CharacterResult::Consume
                }
                None => {
                    let new;

                    if addons.contains(Addon::TreatJongseongAsChoseong) {
                        match jong.to_cho(addons) {
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
                    } else {
                        new = Self {
                            jong: Some(jong),
                            ..Default::default()
                        };
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
        let addons = EnumSet::only(Addon::TreatJongseongAsChoseong);

        state.cho_jong(Choseong::Ieung, Jongseong::Ieung, true, addons);
        state.jung(Jungseong::A, true, addons);
        state.cho_jong(Choseong::Ieung, Jongseong::Ieung, true, addons);

        assert_eq!(
            CharacterResult::NewCharacter(CharacterState {
                cho: Some(Choseong::Ieung),
                jung: Some(Jungseong::A),
                compose_jung: true,
                jong: None
            }),
            state.jung(Jungseong::A, true, addons)
        );
    }

    #[test]
    fn filler() {
        let state = CharacterState {
            cho: Some(Choseong::Nieun),
            jung: None,
            compose_jung: false,
            jong: Some(Jongseong::Digeut),
        };
        let mut out = String::new();
        state.display(true, &mut out);
        assert_eq!(out, "ᄂᅠᆮ");
    }
}
