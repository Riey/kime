mod characters;
mod config;
mod input_result;
mod keycode;
mod state;

mod os;

use ahash::AHashMap;

use crate::characters::KeyValue;

pub use crate::config::{
    Addon, Config, Hotkey, HotkeyBehavior, HotkeyResult, RawConfig, BUILTIN_LAYOUTS,
};
pub use crate::input_result::InputResult;
pub use crate::keycode::{Key, KeyCode, ModifierState};
pub use crate::os::{DefaultOsContext, OsContext};
pub use crate::state::HangulState;

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
    os_ctx: Box<dyn OsContext>,
}

impl Default for InputEngine {
    fn default() -> Self {
        Self::new(true)
    }
}

impl InputEngine {
    pub fn with_os_ctx(word_commit: bool, os_ctx: Box<dyn OsContext>) -> Self {
        Self {
            state: HangulState::new(word_commit),
            enable_hangul: false,
            os_ctx,
        }
    }

    pub fn new(word_commit: bool) -> Self {
        Self::with_os_ctx(word_commit, Box::new(DefaultOsContext::default()))
    }

    pub fn set_hangul_enable(&mut self, enable: bool) {
        self.enable_hangul = enable;
    }

    pub fn is_hangul_enabled(&self) -> bool {
        self.enable_hangul
    }

    pub fn update_hangul_state(&mut self) -> std::io::Result<()> {
        self.os_ctx.update_hangul_state(self.enable_hangul)
    }

    fn bypass(&mut self) -> InputResult {
        self.clear_preedit();
        InputResult::NEED_RESET
    }

    fn check_hangul_state(&mut self, config: &Config) -> bool {
        if config.global_hangul_state {
            self.enable_hangul = self
                .os_ctx
                .read_global_hangul_state()
                .unwrap_or(self.enable_hangul);
        }

        self.enable_hangul
    }

    pub fn press_key(&mut self, key: Key, config: &Config) -> InputResult {
        if let Some(hotkey) = config.hotkeys.get(&key) {
            let mut processed = false;
            let mut ret = InputResult::empty();

            match hotkey.behavior() {
                HotkeyBehavior::ToEnglish => {
                    if self.enable_hangul {
                        self.enable_hangul = false;
                        ret |= InputResult::LANGUAGE_CHANGED;
                        processed = true;
                    }
                }
                HotkeyBehavior::Emoji => {
                    if self.os_ctx.emoji(&mut self.state).unwrap_or(false) {
                        ret |= InputResult::NEED_RESET;
                        processed = true;
                    }
                }
                HotkeyBehavior::Hanja => {
                    if self.os_ctx.hanja(&mut self.state).unwrap_or(false) {
                        ret |= InputResult::NEED_RESET;
                        processed = true;
                    }
                }
                HotkeyBehavior::ToHangul => {
                    if !self.enable_hangul {
                        self.enable_hangul = true;
                        ret |= InputResult::LANGUAGE_CHANGED;
                        processed = true;
                    }
                }
                HotkeyBehavior::ToggleHangul => {
                    self.enable_hangul = !self.enable_hangul;
                    ret |= InputResult::LANGUAGE_CHANGED;
                    processed = true;
                }
                HotkeyBehavior::Commit => {
                    if self
                        .state
                        .preedit_result()
                        .contains(InputResult::HAS_PREEDIT)
                    {
                        self.state.clear_preedit();
                        ret |= InputResult::NEED_RESET;
                        processed = true;
                    }
                }
            }

            match (hotkey.result(), processed) {
                (HotkeyResult::Bypass, _) | (HotkeyResult::ConsumeIfProcessed, false) => {
                    ret |= self.bypass();
                }
                (HotkeyResult::Consume, _) | (HotkeyResult::ConsumeIfProcessed, true) => {
                    ret |= InputResult::CONSUMED | self.state.preedit_result();
                }
            }

            ret
        } else if key.code == KeyCode::Shift {
            // Don't reset state
            self.state.preedit_result()
        } else if self.check_hangul_state(config) {
            if key.code == KeyCode::Backspace {
                self.state.backspace(config)
            } else if let Some(v) = config.layout.keymap.get(&key) {
                self.state.key(v, config)
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
