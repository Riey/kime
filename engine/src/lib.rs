mod characters;
mod config;
mod keycode;
mod state;

use self::characters::{Choseong, Jongseong, Jungseong, KeyValue};
use ahash::AHashMap;
use serde::{Deserialize, Serialize};

pub use self::config::Config;
pub use self::keycode::{Key, KeyCode};
pub use self::state::CharacterState;

#[derive(Clone, Default)]
pub struct Layout {
    keymap: AHashMap<Key, KeyValue>,
}

#[derive(Serialize, Deserialize)]
struct ValueItem {
    #[serde(default)]
    cho: Option<Choseong>,
    #[serde(default)]
    jung: Option<Jungseong>,
    #[serde(default)]
    jong: Option<Jongseong>,
    #[serde(default)]
    pass: Option<char>,
}

impl Layout {
    fn from_items(items: AHashMap<Key, ValueItem>) -> Self {
        let mut keymap = AHashMap::new();

        for (key, value) in items {
            let value = match value {
                ValueItem {
                    pass: Some(pass), ..
                } => KeyValue::Pass(pass),
                ValueItem {
                    cho: Some(cho),
                    jong: Some(jong),
                    ..
                } => KeyValue::ChoJong(cho, jong),
                ValueItem { cho: Some(cho), .. } => KeyValue::Choseong(cho),
                ValueItem {
                    jong: Some(jong), ..
                } => KeyValue::Jongseong(jong),
                ValueItem {
                    jung: Some(jung), ..
                } => KeyValue::Jungseong(jung),
                _ => continue,
            };

            keymap.insert(key, value);
        }

        Self { keymap }
    }

    pub fn load_from(content: &str) -> Option<Self> {
        Some(Self::from_items(serde_yaml::from_str(content).ok()?))
    }

    pub fn map_key(&self, state: &mut CharacterState, key: Key) -> InputResult {
        if key.code == KeyCode::Backspace {
            state.backspace()
        } else {
            if let Some(v) = self.keymap.get(&key) {
                match *v {
                    KeyValue::Pass(pass) => {
                        if let Some(commit) = state.reset() {
                            InputResult::CommitCommit(commit, pass)
                        } else {
                            InputResult::Commit(pass)
                        }
                    }
                    KeyValue::ChoJong(cho, jong) => state.cho_jong(cho, jong),
                    KeyValue::Jungseong(jung) => state.jung(jung),
                    KeyValue::Choseong(cho) => state.cho(cho),
                    KeyValue::Jongseong(jong) => state.jong(jong),
                }
            } else {
                bypass(state)
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputResult {
    ClearPreedit,
    Preedit(char),
    Consume,
    Bypass,
    Commit(char),
    CommitBypass(char),
    /// (commit, preedit)
    CommitPreedit(char, char),
    CommitCommit(char, char),
}

pub struct InputEngine {
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
            InputResult::Consume
        } else if key.code == KeyCode::Esc && config.esc_turn_off {
            self.enable_hangul = false;
            bypass(&mut self.state)
        } else if self.enable_hangul {
            config.layout.map_key(&mut self.state, key)
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
        Some(preedit) => InputResult::CommitBypass(preedit),
        None => InputResult::Bypass,
    }
}
