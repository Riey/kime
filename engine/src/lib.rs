#![allow(clippy::missing_safety_doc)]

mod characters;
mod config;
mod keycode;
mod state;
#[cfg(test)]
mod tests;

use self::characters::KeyValue;
use ahash::AHashMap;
use std::fmt;

use self::config::Config;
use self::keycode::{Key, KeyCode};
use self::state::CharacterState;

#[derive(Clone, Default)]
pub struct Layout {
    keymap: AHashMap<Key, KeyValue>,
}

impl Layout {
    fn from_items(items: AHashMap<Key, String>) -> Self {
        let mut keymap = AHashMap::new();

        for (key, value) in items {
            let value = match value.parse::<KeyValue>() {
                Ok(value) => value,
                Err(_) => continue,
            };

            keymap.insert(key, value);
        }

        Self { keymap }
    }

    #[cfg(test)]
    pub fn load_from(content: &str) -> Result<Self, serde_yaml::Error> {
        Ok(Self::from_items(serde_yaml::from_str(content)?))
    }

    pub(crate) fn map_key(
        &self,
        state: &mut CharacterState,
        config: &Config,
        key: Key,
    ) -> InputResult {
        if key.code == KeyCode::Backspace {
            state.backspace(config)
        } else if let Some(v) = self.keymap.get(&key) {
            match *v {
                KeyValue::Pass(pass) => {
                    if let Some(commit) = state.reset() {
                        InputResult::commit2(commit, pass)
                    } else {
                        InputResult::commit(pass)
                    }
                }
                KeyValue::ChoJong(cho, jong) => state.cho_jong(cho, jong, config),
                KeyValue::Jungseong(jung) => state.jung(jung, config),
                KeyValue::Choseong(cho) => state.cho(cho, config),
                KeyValue::Jongseong(jong) => state.jong(jong, config),
            }
        } else {
            bypass(state)
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputResultType {
    Bypass,
    Consume,
    ClearPreedit,
    Preedit,
    Commit,
    CommitBypass,
    CommitPreedit,
    CommitCommit,
}

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct InputResult {
    pub ty: InputResultType,
    pub char1: u32,
    pub char2: u32,
}

impl fmt::Debug for InputResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("InputResult")
            .field("ty", &self.ty)
            .field("char1", unsafe {
                &std::char::from_u32_unchecked(self.char1)
            })
            .field("char2", unsafe {
                &std::char::from_u32_unchecked(self.char2)
            })
            .finish()
    }
}

impl InputResult {
    pub const fn clear_preedit() -> Self {
        Self {
            ty: InputResultType::ClearPreedit,
            char1: 0,
            char2: 0,
        }
    }

    pub const fn bypass() -> Self {
        Self {
            ty: InputResultType::Bypass,
            char1: 0,
            char2: 0,
        }
    }

    pub const fn consume() -> Self {
        Self {
            ty: InputResultType::Consume,
            char1: 0,
            char2: 0,
        }
    }

    pub const fn preedit(c: char) -> Self {
        Self {
            ty: InputResultType::Preedit,
            char1: c as u32,
            char2: 0,
        }
    }

    pub const fn commit(c: char) -> Self {
        Self {
            ty: InputResultType::Commit,
            char1: c as u32,
            char2: 0,
        }
    }

    pub const fn commit_bypass(c: char) -> Self {
        Self {
            ty: InputResultType::CommitBypass,
            char1: c as u32,
            char2: 0,
        }
    }

    pub const fn commit_preedit(c: char, p: char) -> Self {
        Self {
            ty: InputResultType::CommitPreedit,
            char1: c as u32,
            char2: p as u32,
        }
    }

    pub fn commit2(c1: char, c2: char) -> Self {
        Self {
            ty: InputResultType::CommitCommit,
            char1: c1 as u32,
            char2: c2 as u32,
        }
    }
}

#[derive(Default)]
pub struct InputEngine {
    state: CharacterState,
    enable_hangul: bool,
}

impl InputEngine {
    pub fn new() -> Self {
        Self::default()
    }

    #[cfg(test)]
    pub fn set_enable_hangul(&mut self, enable: bool) {
        self.enable_hangul = enable;
    }

    pub fn press_key(&mut self, key: Key, config: &Config) -> InputResult {
        if config.hangul_keys.contains(&key) {
            self.enable_hangul = !self.enable_hangul;
            InputResult::consume()
        } else if key.code == KeyCode::Shift {
            InputResult::bypass()
        } else if key.code == KeyCode::Esc && config.esc_turn_off {
            self.enable_hangul = false;
            bypass(&mut self.state)
        } else if self.enable_hangul {
            config.layout.map_key(&mut self.state, config, key)
        } else {
            bypass(&mut self.state)
        }
    }

    #[inline]
    pub fn preedit_char(&self) -> char {
        self.state.to_char()
    }

    #[inline]
    pub fn reset(&mut self) -> Option<char> {
        self.state.reset()
    }
}

fn bypass(state: &mut CharacterState) -> InputResult {
    match state.reset() {
        Some(c) => InputResult::commit_bypass(c),
        None => InputResult::bypass(),
    }
}

#[no_mangle]
pub extern "C" fn kime_engine_new() -> *mut InputEngine {
    Box::into_raw(Box::new(InputEngine::new()))
}

#[no_mangle]
pub unsafe extern "C" fn kime_engine_delete(engine: *mut InputEngine) {
    drop(Box::from_raw(engine));
}

#[no_mangle]
pub unsafe extern "C" fn kime_engine_preedit_char(engine: *const InputEngine) -> u32 {
    let engine = engine.as_ref().unwrap();

    engine.preedit_char() as u32
}

#[no_mangle]
pub unsafe extern "C" fn kime_engine_reset(engine: *mut InputEngine) -> u32 {
    let engine = engine.as_mut().unwrap();

    engine.reset().map_or(0, |c| c as u32)
}

#[no_mangle]
pub unsafe extern "C" fn kime_engine_press_key(
    engine: *mut InputEngine,
    config: *const Config,
    hardware_code: u16,
    state: u32,
) -> InputResult {
    match KeyCode::from_hardward_code(hardware_code) {
        Some(code) => {
            let engine = engine.as_mut().unwrap();
            let config = config.as_ref().unwrap();

            engine.press_key(
                Key {
                    code,
                    shift: state & 0x1 != 0,
                    ctrl: state & 0x4 != 0,
                    super_: state & 0x40 != 0,
                },
                config,
            )
        }
        None => InputResult::bypass(),
    }
}

#[no_mangle]
pub extern "C" fn kime_config_load() -> *mut Config {
    Box::into_raw(Box::new(Config::load_from_config_dir().unwrap_or_default()))
}

#[no_mangle]
pub unsafe extern "C" fn kime_config_delete(config: *mut Config) {
    drop(Box::from_raw(config));
}

#[no_mangle]
pub unsafe extern "C" fn kime_config_gtk_commit_english(config: *const Config) -> u32 {
    config.as_ref().unwrap().gtk_commit_english.into()
}

#[no_mangle]
pub unsafe extern "C" fn kime_config_xim_preedit_font(
    config: *const Config,
    name: *mut *const u8,
    len: *mut usize,
) {
    let font = config.as_ref().unwrap().xim_preedit_font.as_str();
    name.write(font.as_ptr());
    len.write(font.len());
}
