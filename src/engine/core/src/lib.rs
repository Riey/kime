mod config;
mod os;

pub use config::{Config, Hotkey, InputCategory, InputMode, RawConfig};
pub use kime_engine_backend::{InputResult, Key, KeyCode, KeyMap, ModifierState};

use config::{HotkeyBehavior, HotkeyResult, IconColor};
use os::{DefaultOsContext, OsContext};

use kime_engine_backend::{InputEngineBackend, InputEngineMode, InputEngineModeResult};
use kime_engine_backend_emoji::EmojiMode;
use kime_engine_backend_hangul::HangulEngine;
use kime_engine_backend_hanja::HanjaMode;
use kime_engine_backend_latin::LatinEngine;
use kime_engine_backend_math::MathMode;

pub struct InputEngine {
    engine_impl: EngineImpl,
    commit_buf: String,
    preedit_buf: String,
    os_ctx: DefaultOsContext,
    icon_color: IconColor,
}

impl Default for InputEngine {
    fn default() -> Self {
        Self::new(&Config::default())
    }
}

impl InputEngine {
    pub fn new(config: &Config) -> Self {
        Self {
            engine_impl: EngineImpl::new(config),
            commit_buf: String::with_capacity(16),
            preedit_buf: String::with_capacity(16),
            os_ctx: DefaultOsContext::default(),
            icon_color: config.icon_color,
        }
    }

    pub fn set_input_category(&mut self, category: InputCategory) {
        // Reset previous engine
        self.engine_impl.clear_preedit(&mut self.commit_buf);
        self.engine_impl.mode = None;
        self.engine_impl.category = category;
    }

    pub fn set_input_mode(&mut self, mode: InputMode) -> bool {
        self.engine_impl.set_mode(mode, &mut self.commit_buf)
    }

    pub fn category(&self) -> InputCategory {
        self.engine_impl.category
    }

    pub fn update_layout_state(&mut self) -> std::io::Result<()> {
        self.os_ctx
            .update_layout_state(self.category(), self.icon_color)
    }

    fn try_get_global_input_category_state(&mut self, config: &Config) {
        if config.global_category_state {
            let global = self
                .os_ctx
                .read_global_hangul_state()
                .unwrap_or(self.category());

            if self.category() != global {
                self.set_input_category(global);
            }
        }
    }

    fn try_hotkey<'c>(&self, key: Key, config: &'c Config) -> Option<&'c Hotkey> {
        if let Some(mode) = self.engine_impl.mode {
            if let mode_hotkey @ Some(_) = config.mode_hotkeys[mode].get(&key) {
                return mode_hotkey;
            }
        } else {
            if let category_hotkey @ Some(_) = config.category_hotkeys[self.category()].get(&key) {
                return category_hotkey;
            }
        }

        config.global_hotkeys.get(&key)
    }

    pub fn press_key(&mut self, key: Key, config: &Config) -> InputResult {
        self.try_get_global_input_category_state(config);

        let mut ret = InputResult::empty();

        if let Some(hotkey) = self.try_hotkey(key, config) {
            let mut processed = false;
            match hotkey.behavior() {
                HotkeyBehavior::Switch(category) => {
                    if self.category() != category || self.engine_impl.mode.is_some() {
                        self.set_input_category(category);
                        ret |= InputResult::LANGUAGE_CHANGED;
                        processed = true;
                    }
                }
                HotkeyBehavior::Toggle(left, right) => {
                    let change = if self.category() == left {
                        right
                    } else if self.category() == right {
                        left
                    } else {
                        right
                    };

                    self.set_input_category(change);
                    ret |= InputResult::LANGUAGE_CHANGED;
                    processed = true;
                }
                HotkeyBehavior::Mode(mode) => {
                    processed = self.engine_impl.set_mode(mode, &mut self.commit_buf);
                }
                HotkeyBehavior::Commit => {
                    if self.engine_impl.has_preedit() {
                        self.engine_impl.clear_preedit(&mut self.commit_buf);
                        processed = true;
                    }
                }
                HotkeyBehavior::Ignore => {
                    processed = true;
                }
            }

            match (hotkey.result(), processed) {
                (HotkeyResult::Bypass, _) | (HotkeyResult::ConsumeIfProcessed, false) => {}
                (HotkeyResult::Consume, _) | (HotkeyResult::ConsumeIfProcessed, true) => {
                    ret |= InputResult::CONSUMED;
                }
            }
        } else if self.engine_impl.press_key(key, &mut self.commit_buf) {
            ret |= InputResult::CONSUMED;
        } else {
            // clear preedit when get unhandled key
            self.clear_preedit();
        }

        ret |= self.current_result();

        ret
    }

    pub fn press_key_code(
        &mut self,
        hardware_code: u16,
        state: ModifierState,
        config: &Config,
    ) -> InputResult {
        match KeyCode::from_hardward_code(hardware_code) {
            Some(code) => self.press_key(Key::new(code, state), config),
            None => self.current_result(),
        }
    }

    #[inline]
    pub fn clear_commit(&mut self) {
        self.commit_buf.clear();
    }

    #[inline]
    pub fn clear_preedit(&mut self) {
        self.engine_impl.clear_preedit(&mut self.commit_buf);
    }

    #[inline]
    pub fn remove_preedit(&mut self) {
        self.engine_impl.reset();
    }

    pub fn preedit_str(&mut self) -> &str {
        self.preedit_buf.clear();
        self.engine_impl.preedit_str(&mut self.preedit_buf);
        &self.preedit_buf
    }

    #[inline]
    pub fn commit_str(&self) -> &str {
        &self.commit_buf
    }

    #[inline]
    pub fn reset(&mut self) {
        self.clear_commit();
        self.remove_preedit();
    }

    fn current_result(&self) -> InputResult {
        let mut ret = InputResult::empty();
        if self.engine_impl.has_preedit() {
            ret |= InputResult::HAS_PREEDIT;
        }
        if !self.commit_buf.is_empty() {
            ret |= InputResult::HAS_COMMIT;
        }
        ret
    }
}

struct EngineImpl {
    category: InputCategory,
    mode: Option<InputMode>,
    latin_engine: LatinEngine,
    hangul_engine: HangulEngine,
    hanja_mode: HanjaMode,
    math_mode: MathMode,
    emoji_mode: EmojiMode,
}

impl EngineImpl {
    pub fn new(config: &Config) -> Self {
        Self {
            category: config.default_category,
            mode: None,
            latin_engine: config.latin_engine.clone(),
            hangul_engine: config.hangul_engine.clone(),
            hanja_mode: HanjaMode::new(),
            math_mode: config.math_mode.clone(),
            emoji_mode: config.emoji_mode.clone(),
        }
    }

    pub fn set_mode(&mut self, mode: InputMode, commit_buf: &mut String) -> bool {
        match mode {
            InputMode::Math | InputMode::Emoji => {
                self.clear_preedit(commit_buf);
                self.mode = Some(mode);
                true
            }
            InputMode::Hanja => match self.category {
                InputCategory::Hangul => {
                    if self.hanja_mode.set_key(self.hangul_engine.get_hanja_char()) {
                        self.hangul_engine.reset();
                        self.mode = Some(InputMode::Hanja);
                        true
                    } else {
                        false
                    }
                }
                _ => false,
            },
        }
    }
}

macro_rules! do_mode {
    (@retarm $field:ident $self:expr, $func:ident($($arg:expr,)*)) => {
        match $self.$field.$func($($arg,)*) {
            InputEngineModeResult::Continue(ret) => {
                return ret;
            }
            InputEngineModeResult::Exit => {
                $self.mode = None;
            }
        }
    };
    (@ret $self:expr, $func:ident($($arg:expr,)*)) => {
        match $self.mode {
            Some(InputMode::Math) => {
                do_mode!(@retarm math_mode $self, $func($($arg,)*));
            }
            Some(InputMode::Hanja) => {
                do_mode!(@retarm hanja_mode $self, $func($($arg,)*));
            }
            Some(InputMode::Emoji) => {
                do_mode!(@retarm emoji_mode $self, $func($($arg,)*));
            }
            None => {}
        }
    };
    (@direct $self:expr, $func:ident($($arg:expr,)*)) => {
        match $self.mode {
            Some(InputMode::Hanja) => {
                return $self.hanja_mode.$func($($arg,)*);
            }
            Some(InputMode::Math) => {
                return $self.math_mode.$func($($arg,)*);
            }
            Some(InputMode::Emoji) => {
                return $self.emoji_mode.$func($($arg,)*);
            }
            None => {}
        }
    };
}

macro_rules! do_engine {
    ($self:expr, $func:ident($($arg:expr,)*)) => {
        match $self.category {
            InputCategory::Hangul => $self.hangul_engine.$func($($arg,)*),
            InputCategory::Latin => $self.latin_engine.$func($($arg,)*),
        }
    };
}

macro_rules! connect {
    (@$key:ident $self:expr, $func:ident($($arg:expr$(,)?)*)) => {{
        do_mode!(@$key $self, $func($($arg,)*));
        do_engine!($self, $func($($arg,)*))
    }};
}

impl InputEngineBackend for EngineImpl {
    fn press_key(&mut self, key: Key, commit_buf: &mut String) -> bool {
        connect!(@ret self, press_key(key, commit_buf))
    }

    fn clear_preedit(&mut self, commit_buf: &mut String) {
        connect!(@ret self, clear_preedit(commit_buf));
    }

    fn reset(&mut self) {
        connect!(@ret self, reset());
    }

    fn has_preedit(&self) -> bool {
        connect!(@direct self, has_preedit())
    }

    fn preedit_str(&self, buf: &mut String) {
        connect!(@direct self, preedit_str(buf));
    }
}
