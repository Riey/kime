mod config;
mod os;

use config::{HotkeyBehavior, HotkeyResult, IconColor};

use os::{DefaultOsContext, OsContext};

use kime_engine_core::InputEngine as _;
use kime_engine_hangul::{HangulConfig, HangulEngine};
use kime_engine_latin::{LatinConfig, LatinEngine};

pub use config::{Config, InputCategory, RawConfig};

pub use kime_engine_core::{InputResult, Key, KeyCode, ModifierState};

pub struct InputEngine {
    engine_impl: EngineImpl,
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
            preedit_buf: String::with_capacity(16),
            os_ctx: DefaultOsContext::default(),
            icon_color: config.icon_color,
        }
    }

    pub fn set_input_category(&mut self, category: InputCategory) {
        // Reset previous engine
        self.engine_impl.reset();
        self.engine_impl.category = category;
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

    pub fn press_key(&mut self, key: Key, config: &Config) -> InputResult {
        if let Some(hotkey) = config.hotkeys.get(&key) {
            let mut processed = false;
            let mut ret = InputResult::empty();

            match hotkey.behavior() {
                HotkeyBehavior::Switch(category) => {
                    if self.category() != category {
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
                HotkeyBehavior::Emoji => {
                    if self.os_ctx.emoji(&mut self.engine_impl).unwrap_or(false) {
                        processed = true;
                    }
                }
                HotkeyBehavior::Hanja => {
                    if self.os_ctx.hanja(&mut self.engine_impl).unwrap_or(false) {
                        processed = true;
                    }
                }
                HotkeyBehavior::Commit => {
                    if self.engine_impl.has_preedit() {
                        self.engine_impl.clear_preedit();
                        processed = true;
                    }
                }
            }

            match (hotkey.result(), processed) {
                (HotkeyResult::Bypass, _) | (HotkeyResult::ConsumeIfProcessed, false) => {}
                (HotkeyResult::Consume, _) | (HotkeyResult::ConsumeIfProcessed, true) => {
                    ret |= InputResult::CONSUMED;
                }
            }

            ret |= self.engine_impl.current_result();

            ret
        } else if key.code == KeyCode::Shift {
            // Don't reset state
            self.engine_impl.current_result()
        } else {
            self.try_get_global_input_category_state(config);

            let mut ret = self.engine_impl.current_result();

            if self.engine_impl.press_key(key) {
                ret |= InputResult::CONSUMED;
            }

            ret
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
            None => self.engine_impl.current_result(),
        }
    }

    #[inline]
    pub fn clear_preedit(&mut self) {
        self.engine_impl.clear_preedit();
    }

    #[inline]
    pub fn remove_preedit(&mut self) {
        self.engine_impl.remove_preedit();
    }

    #[inline]
    pub fn preedit_str(&mut self) -> &str {
        self.preedit_buf.clear();
        self.engine_impl.preedit_str(&mut self.preedit_buf);
        &self.preedit_buf
    }

    #[inline]
    pub fn commit_str(&self) -> &str {
        self.engine_impl.commit_str()
    }

    #[inline]
    pub fn reset(&mut self) {
        self.engine_impl.reset();
    }
}

struct EngineImpl {
    category: InputCategory,
    latin_engine: LatinEngine,
    hangul_engine: HangulEngine,
    math_engine: LatinEngine,
}

impl EngineImpl {
    pub fn new(config: &Config) -> Self {
        Self {
            category: config.default_category,
            latin_engine: config.latin_engine.clone(),
            hangul_engine: config.hangul_engine.clone(),
            math_engine: config.latin_engine.clone(),
        }
    }

    pub fn current_result(&self) -> InputResult {
        let mut ret = InputResult::empty();
        if self.has_preedit() {
            ret |= InputResult::HAS_PREEDIT;
        }
        if !self.commit_str().is_empty() {
            ret |= InputResult::HAS_COMMIT;
        }
        ret
    }
}

macro_rules! do_engine {
    ($self:expr, $func:ident($($arg:expr),*)) => {
        match $self.category {
            InputCategory::Hangul => $self.hangul_engine.$func($($arg,)*),
            InputCategory::Latin => $self.latin_engine.$func($($arg,)*),
            InputCategory::Math => $self.math_engine.$func($($arg,)*),
        }
    };
}

impl kime_engine_core::InputEngine for EngineImpl {
    fn press_key(&mut self, key: Key) -> bool {
        do_engine!(self, press_key(key))
    }

    fn clear_preedit(&mut self) {
        do_engine!(self, clear_preedit());
    }

    fn clear_commit(&mut self) {
        do_engine!(self, clear_commit());
    }

    fn reset(&mut self) {
        do_engine!(self, reset());
    }

    fn has_preedit(&self) -> bool {
        do_engine!(self, has_preedit())
    }

    fn preedit_str(&self, buf: &mut String) {
        do_engine!(self, preedit_str(buf));
    }

    fn commit_str(&self) -> &str {
        do_engine!(self, commit_str())
    }

    fn pass(&mut self, s: &str) {
        do_engine!(self, pass(s));
    }
}
