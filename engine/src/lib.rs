mod characters;
mod config;
mod keycode;
mod state;
mod tests;

use self::characters::KeyValue;
use ahash::AHashMap;

use self::config::{Config, RawConfig};
use self::keycode::{Key, KeyCode};
use self::state::CharacterState;

use std::{num::NonZeroU32, os::raw::c_void};

#[derive(Clone, Default)]
struct Layout {
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

    pub fn load_from(content: &str) -> Result<Self, serde_yaml::Error> {
        Ok(Self::from_items(serde_yaml::from_str(content)?))
    }

    pub fn map_key(&self, state: &mut CharacterState, config: &Config, key: Key) -> InputResult {
        if key.code == KeyCode::Backspace {
            state.backspace(config)
        } else {
            if let Some(v) = self.keymap.get(&key) {
                match *v {
                    KeyValue::Pass(pass) => {
                        if let Some(commit) = state.reset() {
                            InputResult::commit2(commit, pass)
                        } else {
                            InputResult::commit(pass)
                        }
                    }
                    KeyValue::ChoJong(cho, jong) => state.cho_jong(cho, jong, config),
                    KeyValue::Jungseong(jung) => state.jung(jung),
                    KeyValue::Choseong(cho) => state.cho(cho, config),
                    KeyValue::Jongseong(jong) => state.jong(jong, config),
                }
            } else {
                bypass(state)
            }
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct InputResult {
    pub bypass: u8,
    pub clear_preedit: u8,
    pub preedit: Option<NonZeroU32>,
    pub commit_1: Option<NonZeroU32>,
    pub commit_2: Option<NonZeroU32>,
}

impl InputResult {
    pub const fn new() -> Self {
        Self {
            bypass: 0,
            clear_preedit: 0,
            preedit: None,
            commit_1: None,
            commit_2: None,
        }
    }

    pub const fn clear_preedit() -> Self {
        Self {
            clear_preedit: 1,
            ..Self::new()
        }
    }

    pub const fn bypass() -> Self {
        Self {
            bypass: 1,
            ..Self::new()
        }
    }

    pub const fn consume() -> Self {
        Self { ..Self::new() }
    }

    pub const fn preedit(c: char) -> Self {
        Self {
            preedit: NonZeroU32::new(c as u32),
            ..Self::new()
        }
    }

    pub const fn commit(c: char) -> Self {
        Self {
            commit_1: NonZeroU32::new(c as u32),
            ..Self::new()
        }
    }

    pub const fn commit_bypass(c: char) -> Self {
        Self {
            commit_1: NonZeroU32::new(c as u32),
            bypass: 1,
            ..Self::new()
        }
    }

    pub const fn commit_preedit(c: char, p: char) -> Self {
        Self {
            commit_1: NonZeroU32::new(c as u32),
            preedit: NonZeroU32::new(p as u32),
            ..Self::new()
        }
    }

    pub fn commit2(c1: char, c2: char) -> Self {
        Self {
            commit_1: NonZeroU32::new(c1 as u32),
            commit_2: NonZeroU32::new(c2 as u32),
            ..Self::new()
        }
    }
}

struct InputEngine {
    state: CharacterState,
    enable_hangul: bool,
}

impl InputEngine {
    pub fn new() -> Self {
        Self {
            state: CharacterState::default(),
            enable_hangul: false,
        }
    }

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
pub extern "C" fn kime_engine_new() -> *mut c_void {
    Box::into_raw(Box::new(InputEngine::new())).cast()
}

#[no_mangle]
pub unsafe extern "C" fn kime_engine_delete(engine: *mut c_void) {
    drop(Box::from_raw(engine.cast::<InputEngine>()));
}

#[no_mangle]
pub unsafe extern "C" fn kime_engine_set_enable_hangul(engine: *mut c_void, enable: u32) {
    let engine = engine.cast::<InputEngine>().as_mut().unwrap();

    engine.set_enable_hangul(enable != 0);
}

#[no_mangle]
pub unsafe extern "C" fn kime_engine_reset(engine: *mut c_void) -> u32 {
    let engine = engine.cast::<InputEngine>().as_mut().unwrap();

    engine.reset().map_or(0, |c| c as u32)
}

#[no_mangle]
pub unsafe extern "C" fn kime_engine_press_key(
    engine: *mut c_void,
    config: *const c_void,
    hardware_code: u16,
    state: u32,
) -> InputResult {
    match KeyCode::from_hardward_code(hardware_code) {
        Some(code) => {
            let engine = engine.cast::<InputEngine>().as_mut().unwrap();
            let config = config.cast::<Config>().as_ref().unwrap();

            engine.press_key(
                Key {
                    code,
                    shift: state & 0x1 != 0,
                    ctrl: state & 0x4 != 0,
                },
                config,
            )
        }
        None => InputResult::bypass(),
    }
}

#[no_mangle]
pub extern "C" fn kime_config_load() -> *mut c_void {
    Box::into_raw(Box::new(Config::load_from_config_dir().unwrap_or_default())).cast()
}

#[no_mangle]
pub unsafe extern "C" fn kime_config_delete(config: *mut c_void) {
    drop(Box::from_raw(config.cast::<Config>()));
}

#[no_mangle]
pub unsafe extern "C" fn kime_config_gtk_commit_english(config: *mut c_void) -> u32 {
    config
        .cast::<Config>()
        .as_mut()
        .unwrap()
        .gtk_commit_english
        .into()
}

#[no_mangle]
pub unsafe extern "C" fn kime_config_xim_preedit_font(
    config: *mut c_void,
    name: *mut *const u8,
    len: *mut usize,
) {
    let font = config
        .cast::<Config>()
        .as_mut()
        .unwrap()
        .xim_preedit_font
        .as_str();
    name.write(font.as_ptr());
    len.write(font.len());
}
