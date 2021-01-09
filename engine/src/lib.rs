mod characters;
mod dubeolsik;
mod keycode;
mod state;

use self::{
    characters::{Choseong, Jongseong, Jungseong, KeyValue},
    keycode::{Key, KeyCode},
};
use ahash::AHashMap;
use serde::{Deserialize, Serialize};

pub use self::state::CharacterState;

pub struct Layout {
    keymap: AHashMap<Key, KeyValue>,
}

#[derive(Serialize, Deserialize)]
struct KeyItem {
    code: KeyCode,
    // None = both
    #[serde(default)]
    shift: Option<bool>,
    #[serde(flatten)]
    value: ValueItem,
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
    pass: Option<char>
}

impl Layout {
    pub fn dubeolsik() -> Self {
        Self::load_from(self::dubeolsik::DUBEOLSIK_LAYOUT)
    }

    pub fn load_from(content: &str) -> Self {
        let mut keymap = AHashMap::new();

        let items: Vec<KeyItem> = serde_yaml::from_str(content).unwrap();

        for item in items {
            let value = match item.value {
                ValueItem { pass: Some(pass), .. } => KeyValue::Pass(pass),
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

            if let Some(shift) = item.shift {
                keymap.insert(
                    Key {
                        code: item.code,
                        shift,
                    },
                    value,
                );
            } else {
                keymap.insert(
                    Key {
                        code: item.code,
                        shift: true,
                    },
                    value,
                );
                keymap.insert(
                    Key {
                        code: item.code,
                        shift: false,
                    },
                    value,
                );
            }
        }

        Self { keymap }
    }

    pub fn map_key(
        &self,
        state: &mut CharacterState,
        enable_hangul: &mut bool,
        keycode: KeyCode,
        shift: bool,
    ) -> InputResult {
        if keycode == KeyCode::Bs {
            state.backspace()
        } else if matches!(keycode, KeyCode::Henkan | KeyCode::Ralt) {
            *enable_hangul = !*enable_hangul;
            InputResult::Consume
        } else {
            if !*enable_hangul {
                InputResult::Bypass
            } else if let Some(v) = self.keymap.get(&Key {
                code: keycode,
                shift,
            }) {
                match *v {
                    KeyValue::Pass(pass) => {
                        if let Some(preedit) = state.preedit_char() {
                            state.reset();
                            InputResult::CommitCommit(preedit, pass)
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
                InputResult::Bypass
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputResult {
    ClearPreedit,
    Preedit(char),
    // Commit(char),
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
    layout: Layout,
    enable_hangul: bool,
}

impl InputEngine {
    pub fn new(layout: Layout) -> Self {
        Self {
            state: CharacterState::default(),
            layout,
            enable_hangul: false,
        }
    }

    fn bypass(&mut self) -> InputResult {
        if let Some(ch) = self.state.preedit_char() {
            self.state.reset();
            InputResult::CommitBypass(ch)
        } else {
            InputResult::Bypass
        }
    }

    pub fn key_press(&mut self, keycode: u8, shift: bool, ctrl: bool) -> InputResult {
        // skip ctrl
        if ctrl {
            return self.bypass();
        }

        if let Some(keycode) = KeyCode::from_hardware_code(keycode) {
            self.layout
                .map_key(&mut self.state, &mut self.enable_hangul, keycode, shift)
        } else {
            self.bypass()
        }
    }

    #[inline]
    pub fn preedit_char(&self) -> Option<char> {
        self.state.preedit_char()
    }

    #[inline]
    pub fn reset(&mut self) -> Option<char> {
        self.state.reset()
    }
}
