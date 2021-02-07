mod characters;
mod config;
mod input_result;
mod keycode;
mod state;

use std::io::Read;

use self::characters::KeyValue;
use self::state::CharacterState;
use ahash::AHashMap;

pub use self::config::{Config, RawConfig};
pub use self::input_result::{InputResult, InputResultType};
pub use self::keycode::{Key, KeyCode, ModifierState};

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

pub struct InputEngine {
    state: CharacterState,
    enable_hangul: bool,
}

impl Default for InputEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl InputEngine {
    pub fn new() -> Self {
        Self {
            state: CharacterState::default(),
            enable_hangul: false,
        }
    }

    pub fn set_hangul_enable(&mut self, enable: bool) {
        self.enable_hangul = enable;
    }

    pub fn is_hangul_enabled(&self) -> bool {
        self.enable_hangul
    }

    fn read_global_hangul_state(&self) -> std::io::Result<bool> {
        let mut file = std::fs::File::open("/tmp/kime_hangul_state")?;
        let mut buf = [0; 1];
        file.read_exact(&mut buf)?;
        Ok(buf[0] != b'0')
    }

    fn check_hangul_state(&mut self, config: &Config) -> bool {
        if config.global_hangul_state {
            self.enable_hangul = self
                .read_global_hangul_state()
                .unwrap_or(self.enable_hangul);
        }

        self.enable_hangul
    }

    pub fn update_hangul_state(&mut self) {
        std::fs::write(
            "/tmp/kime_hangul_state",
            if self.enable_hangul { "1" } else { "0" },
        )
        .ok();
    }

    pub fn press_key(&mut self, key: Key, config: &Config) -> InputResult {
        if config.hangul_keys.contains(&key) {
            self.enable_hangul = !self.enable_hangul;
            InputResult::toggle_hangul()
        } else if key.code == KeyCode::Shift {
            // Don't reset state
            InputResult::bypass()
        } else if key.code == KeyCode::Esc && config.esc_turn_off {
            self.enable_hangul = false;
            self.bypass()
        } else if self.check_hangul_state(config) {
            if key.code == KeyCode::Backspace {
                self.state.backspace(config)
            } else if let Some(v) = config.layout.keymap.get(&key) {
                match *v {
                    KeyValue::Pass(pass) => match self.state.reset() {
                        '\0' => InputResult::commit(pass),
                        commit => InputResult::commit2(commit, pass),
                    },
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
        state: ModifierState,
        config: &Config,
    ) -> InputResult {
        match KeyCode::from_hardward_code(hardware_code) {
            Some(code) => self.press_key(Key::new(code, state), config),
            None => self.bypass(),
        }
    }

    fn bypass(&mut self) -> InputResult {
        match self.state.reset() {
            '\0' => InputResult::bypass(),
            c => InputResult::commit_bypass(c),
        }
    }

    #[inline]
    pub fn preedit_char(&self) -> char {
        self.state.to_char()
    }

    #[inline]
    pub fn reset(&mut self) -> char {
        self.state.reset()
    }
}
