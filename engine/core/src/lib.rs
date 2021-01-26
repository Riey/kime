mod characters;
mod config;
mod input_result;
mod keycode;
mod state;

use ahash::AHashMap;
use notify_rust::{Notification, NotificationHandle};

use self::characters::KeyValue;
use self::state::CharacterState;

pub use self::config::{Config, ModuleType, RawConfig};
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

#[derive(Default)]
pub struct InputEngine {
    state: CharacterState,
    module_ty: ModuleType,
    notification: Option<NotificationHandle>,
    enable_hangul: bool,
}

impl InputEngine {
    pub fn new() -> Self {
        Self::with_module_ty(ModuleType::default())
    }

    pub fn with_module_ty(module_ty: ModuleType) -> Self {
        Self {
            module_ty,
            ..Default::default()
        }
    }

    pub fn set_enable_hangul(&mut self, enable: bool) {
        self.enable_hangul = enable;
    }

    fn clear_notify(&mut self) {
        if let Some(handle) = self.notification.take() {
            handle.close();
        }
    }

    fn update_preedit(&mut self, ch: char) {
        if let Some(handle) = self.notification.as_mut() {
            handle.body = ch.to_string();
            handle.update();
        } else {
            self.notification = Notification::new()
                .hint(notify_rust::Hint::Urgency(notify_rust::Urgency::Low))
                .summary(&ch.to_string())
                .timeout(2000)
                .show()
                .ok();
        }
    }

    pub fn press_key(&mut self, key: Key, config: &Config) -> InputResult {
        let ret = self.press_key_impl(key, config);

        if config.notify_modules.contains(&self.module_ty) {
            self.clear_notify();
            match ret.ty {
                InputResultType::ClearPreedit
                | InputResultType::CommitBypass => self.clear_notify(),
                InputResultType::Preedit => {
                    self.update_preedit(unsafe { std::char::from_u32_unchecked(ret.char1) });
                }
                InputResultType::CommitPreedit => {
                    self.update_preedit(unsafe { std::char::from_u32_unchecked(ret.char2) });
                }
                InputResultType::Consume => {
                    self.clear_notify();
                    self.notification = Notification::new()
                        .summary(if self.enable_hangul {
                            "Hangul"
                        } else {
                            "English"
                        })
                        .timeout(1000)
                        .show()
                        .ok();
                }
                InputResultType::Bypass | InputResultType::Commit | InputResultType::CommitCommit => {}
            }
        }

        ret
    }

    fn press_key_impl(&mut self, key: Key, config: &Config) -> InputResult {
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
