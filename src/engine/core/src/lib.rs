mod characters;
mod config;
mod input_result;
mod keycode;
mod state;

use ahash::AHashMap;
use std::io::Read;

use self::characters::KeyValue;
use self::config::{HotkeyBehavior, HotkeyResult};
use self::state::HangulState;

pub use self::config::{Config, RawConfig};
pub use self::input_result::InputResult;
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
    state: HangulState,
    enable_hangul: bool,
}

impl Default for InputEngine {
    fn default() -> Self {
        Self::new(true)
    }
}

impl InputEngine {
    pub fn new(word_commit: bool) -> Self {
        Self {
            state: HangulState::new(word_commit),
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

    fn bypass(&mut self) -> InputResult {
        self.clear_preedit();
        InputResult::NEED_RESET
    }

    pub fn update_hangul_state(&mut self) {
        std::fs::write(
            "/tmp/kime_hangul_state",
            if self.enable_hangul { "1" } else { "0" },
        )
        .ok();
    }

    pub fn press_key(&mut self, key: Key, config: &Config) -> InputResult {
        if let Some(hotkey) = config.hotkeys.get(&key) {
            let first = self.enable_hangul;

            match hotkey.behavior() {
                HotkeyBehavior::ToEnglish => {
                    self.enable_hangul = false;
                }
                HotkeyBehavior::ToHangul => {
                    self.enable_hangul = true;
                }
                HotkeyBehavior::ToggleHangul => {
                    self.enable_hangul = !self.enable_hangul;
                }
            }

            let mut ret = match hotkey.result() {
                HotkeyResult::Bypass => self.bypass(),
                HotkeyResult::Consume => InputResult::CONSUMED,
            };

            if self.enable_hangul != first {
                ret.insert(InputResult::LANGUAGE_CHANGED);
            }

            ret
        } else if key.code == KeyCode::Shift {
            // Don't reset state
            self.state.preedit_result()
        } else if self.check_hangul_state(config) {
            if key.code == KeyCode::Backspace {
                self.state.backspace(config)
            } else if let Some(v) = config.layout.keymap.get(&key) {
                match *v {
                    KeyValue::Pass(ref pass) => {
                        self.state.pass(pass);
                        InputResult::NEED_RESET | InputResult::CONSUMED
                    }
                    KeyValue::ChoJong(cho, jong, first) => {
                        self.state.cho_jong(cho, jong, first, config)
                    }
                    KeyValue::ChoJung(cho, jung, first) => {
                        self.state.cho_jung(cho, jung, first, config)
                    }
                    KeyValue::JungJong(jung, jong, first) => {
                        self.state.jung_jong(jung, jong, first, config)
                    }
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

    #[inline]
    pub fn clear_preedit(&mut self) {
        self.state.clear_preedit();
    }

    #[inline]
    pub fn preedit_str(&mut self) -> &str {
        self.state.preedit_str()
    }

    #[inline]
    pub fn commit_str(&mut self) -> &str {
        self.state.commit_str()
    }

    #[inline]
    pub fn flush(&mut self) {
        self.state.flush();
    }

    #[inline]
    pub fn reset(&mut self) {
        self.state.reset();
    }
}
