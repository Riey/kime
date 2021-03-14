mod characters;
mod config;
mod input_result;
mod keycode;
mod state;

mod os;

use ahash::AHashMap;

use crate::characters::KeyValue;
use crate::os::{DefaultOsContext, OsContext};
use enumset::EnumSet;

pub use crate::config::{
    Addon, Config, Hotkey, HotkeyBehavior, HotkeyResult, IconColor, InputCategory, RawConfig,
    BUILTIN_LAYOUTS,
};
pub use crate::input_result::InputResult;
pub use crate::keycode::{Key, KeyCode, ModifierState};
pub use crate::state::HangulState;

pub struct LayoutContext {
    pub input_category: InputCategory,
    pub layout_addons: EnumSet<Addon>,
}

impl Default for LayoutContext {
    fn default() -> Self {
        Self::new(&Config::default())
    }
}

impl LayoutContext {
    pub fn new(config: &Config) -> Self {
        Self {
            input_category: config.default_category,
            layout_addons: config.layout_addons[config.default_category],
        }
    }

    pub fn update_layout(&mut self, category: InputCategory, config: &Config) {
        self.input_category = category;
        self.layout_addons = config.layout_addons[category];
    }

    pub fn check_addon(&self, addon: Addon) -> bool {
        self.layout_addons.contains(addon)
    }
}

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
    layout_ctx: LayoutContext,
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
            state: HangulState::new(config.word_commit),
            os_ctx: DefaultOsContext::default(),
            layout_ctx: LayoutContext::new(config),
            icon_color: config.icon_color,
        }
    }

    pub fn set_input_category(&mut self, config: &Config, category: InputCategory) {
        self.layout_ctx.update_layout(category, config);
    }

    pub fn category(&self) -> InputCategory {
        self.layout_ctx.input_category
    }

    pub fn update_layout_state(&mut self) -> std::io::Result<()> {
        self.os_ctx
            .update_layout_state(self.layout_ctx.input_category, self.icon_color)
    }

    fn bypass(&mut self) -> InputResult {
        self.clear_preedit();
        InputResult::NEED_RESET
    }

    fn try_get_global_input_category_state(&mut self, config: &Config) {
        if config.global_category_state {
            let global = self
                .os_ctx
                .read_global_hangul_state()
                .unwrap_or(self.category());

            if self.layout_ctx.input_category != global {
                self.layout_ctx.update_layout(global, config);
            }
        }
    }

    pub fn press_key(&mut self, key: Key, config: &Config) -> InputResult {
        if let Some(hotkey) = config.hotkeys.get(&key) {
            let mut processed = false;
            let mut ret = InputResult::empty();

            match hotkey.behavior() {
                HotkeyBehavior::Switch(category) => {
                    if self.layout_ctx.input_category != category {
                        self.layout_ctx.update_layout(category, config);
                        ret |= InputResult::LANGUAGE_CHANGED;
                        processed = true;
                    }
                }
                HotkeyBehavior::Toggle(left, right) => {
                    let change = if self.layout_ctx.input_category == left {
                        right
                    } else if self.layout_ctx.input_category == right {
                        left
                    } else {
                        right
                    };

                    self.layout_ctx.update_layout(change, config);
                    ret |= InputResult::LANGUAGE_CHANGED;
                    processed = true;
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
        } else {
            self.try_get_global_input_category_state(config);

            if key.code == KeyCode::Backspace {
                self.state.backspace(&self.layout_ctx)
            } else if let Some(v) = config.layouts[self.layout_ctx.input_category]
                .keymap
                .get(&key)
            {
                self.state.key(v, &self.layout_ctx)
            } else {
                self.bypass()
            }
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
