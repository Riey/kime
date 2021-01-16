mod characters;
mod config;
mod input_result;
mod keycode;
mod state;

use self::characters::KeyValue;
use self::state::CharacterState;
use ahash::AHashMap;

pub use self::config::{Config, RawConfig};
pub use self::input_result::{InputResult, InputResultType};
pub use self::keycode::{Key, KeyCode};

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

    pub fn load_from(content: &str) -> Result<Self, serde_yaml::Error> {
        Ok(Self::from_items(serde_yaml::from_str(content)?))
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

    pub fn set_enable_hangul(&mut self, enable: bool) {
        self.enable_hangul = enable;
    }

    pub fn press_key(&mut self, key: Key, config: &Config) -> InputResult {
        if config.hangul_keys.contains(&key) {
            self.enable_hangul = !self.enable_hangul;
            InputResult::consume()
        } else if key.code == KeyCode::Shift {
            // Don't reset state
            InputResult::bypass()
        } else if key.code == KeyCode::Esc && config.esc_turn_off {
            self.enable_hangul = false;
            self.bypass()
        } else if self.enable_hangul {
            if key.code == KeyCode::Backspace {
                self.state.backspace(config)
            } else if let Some(v) = config.layout.keymap.get(&key) {
                match *v {
                    KeyValue::Pass(pass) => {
                        if let Some(commit) = self.state.reset() {
                            InputResult::commit2(commit, pass)
                        } else {
                            InputResult::commit(pass)
                        }
                    }
                    KeyValue::ChoJong(cho, jong) => self.state.cho_jong(cho, jong, config),
                    KeyValue::Jungseong(jung) => self.state.jung(jung, config),
                    KeyValue::Choseong(cho) => self.state.cho(cho, config),
                    KeyValue::Jongseong(jong) => self.state.jong(jong, config),
                }
            } else {
                self.bypass()
            }
        } else {
            self.bypass()
        }
    }

    pub fn press_key_code(
        &mut self,
        hardware_code: u16,
        state: u32,
        config: &Config,
    ) -> InputResult {
        match KeyCode::from_hardward_code(hardware_code) {
            Some(code) => self.press_key(
                Key {
                    code,
                    shift: state & 0x1 != 0,
                    ctrl: state & 0x4 != 0,
                    super_: state & 0x40 != 0,
                },
                config,
            ),
            None => self.bypass(),
        }
    }

    fn bypass(&mut self) -> InputResult {
        match self.state.reset() {
            Some(c) => InputResult::commit_bypass(c),
            None => InputResult::bypass(),
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
