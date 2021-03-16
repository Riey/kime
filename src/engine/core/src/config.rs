use crate::{HangulConfig, HangulEngine, LatinConfig, LatinEngine};
use enumset::{EnumSet, EnumSetType};
use kime_engine_backend::{AHashMap, Key, KeyCode, ModifierState};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Serialize, Deserialize, Debug, EnumSetType)]
#[enumset(serialize_as_list)]
#[repr(u32)]
pub enum InputCategory {
    Latin,
    Hangul,
    Math,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub enum HotkeyBehavior {
    Toggle(InputCategory, InputCategory),
    Switch(InputCategory),
    Commit,
    Emoji,
    Hanja,
}

impl HotkeyBehavior {
    pub const fn toggle_hangul_latin() -> Self {
        Self::Toggle(InputCategory::Hangul, InputCategory::Latin)
    }

    pub const fn switch(category: InputCategory) -> Self {
        Self::Switch(category)
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub enum HotkeyResult {
    Consume,
    Bypass,
    ConsumeIfProcessed,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct Hotkey {
    behavior: HotkeyBehavior,
    #[serde(default = "EnumSet::all")]
    category: EnumSet<InputCategory>,
    result: HotkeyResult,
}

impl Hotkey {
    pub const fn with_category(
        behavior: HotkeyBehavior,
        category: EnumSet<InputCategory>,
        result: HotkeyResult,
    ) -> Self {
        Self {
            behavior,
            category,
            result,
        }
    }

    pub fn new(behavior: HotkeyBehavior, result: HotkeyResult) -> Self {
        Self::with_category(behavior, EnumSet::all(), result)
    }

    pub const fn behavior(self) -> HotkeyBehavior {
        self.behavior
    }
    pub const fn result(self) -> HotkeyResult {
        self.result
    }
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum IconColor {
    White,
    Black,
}

impl Default for IconColor {
    fn default() -> Self {
        Self::Black
    }
}

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct RawConfig {
    pub default_category: InputCategory,
    pub global_category_state: bool,
    pub icon_color: IconColor,
    pub hotkeys: BTreeMap<Key, Hotkey>,
    pub xim_preedit_font: (String, f64),
    pub latin: LatinConfig,
    pub hangul: HangulConfig,
}

impl Default for RawConfig {
    fn default() -> Self {
        Self {
            latin: LatinConfig::default(),
            hangul: HangulConfig::default(),
            default_category: InputCategory::Latin,
            global_category_state: false,
            icon_color: IconColor::default(),
            hotkeys: [
                (
                    Key::normal(KeyCode::Esc),
                    Hotkey::new(
                        HotkeyBehavior::switch(InputCategory::Latin),
                        HotkeyResult::Bypass,
                    ),
                ),
                (
                    Key::normal(KeyCode::AltR),
                    Hotkey::new(HotkeyBehavior::toggle_hangul_latin(), HotkeyResult::Consume),
                ),
                (
                    Key::normal(KeyCode::Muhenkan),
                    Hotkey::new(HotkeyBehavior::toggle_hangul_latin(), HotkeyResult::Consume),
                ),
                (
                    Key::normal(KeyCode::Hangul),
                    Hotkey::new(HotkeyBehavior::toggle_hangul_latin(), HotkeyResult::Consume),
                ),
                (
                    Key::super_(KeyCode::Space),
                    Hotkey::new(HotkeyBehavior::toggle_hangul_latin(), HotkeyResult::Consume),
                ),
                (
                    Key::normal(KeyCode::Enter),
                    Hotkey::with_category(
                        HotkeyBehavior::Commit,
                        EnumSet::only(InputCategory::Math),
                        HotkeyResult::ConsumeIfProcessed,
                    ),
                ),
                (
                    Key::normal(KeyCode::F9),
                    Hotkey::new(HotkeyBehavior::Hanja, HotkeyResult::Consume),
                ),
                (
                    Key::new(KeyCode::E, ModifierState::CONTROL | ModifierState::ALT),
                    Hotkey::new(HotkeyBehavior::Emoji, HotkeyResult::ConsumeIfProcessed),
                ),
                (
                    Key::normal(KeyCode::ControlR),
                    Hotkey::new(HotkeyBehavior::Hanja, HotkeyResult::Consume),
                ),
                (
                    Key::normal(KeyCode::HangulHanja),
                    Hotkey::new(HotkeyBehavior::Hanja, HotkeyResult::Consume),
                ),
            ]
            .iter()
            .copied()
            .collect(),
            xim_preedit_font: ("D2Coding".to_string(), 15.0),
        }
    }
}

pub struct Config {
    pub default_category: InputCategory,
    pub global_category_state: bool,
    pub hotkeys: AHashMap<Key, Hotkey>,
    pub icon_color: IconColor,
    pub xim_preedit_font: (String, f64),
    pub hangul_engine: HangulEngine,
    pub latin_engine: LatinEngine,
}

impl Default for Config {
    fn default() -> Self {
        Self::new(RawConfig::default())
    }
}

impl Config {
    pub fn new(raw: RawConfig) -> Self {
        Self {
            default_category: raw.default_category,
            global_category_state: raw.global_category_state,
            hotkeys: raw.hotkeys.into_iter().collect(),
            icon_color: raw.icon_color,
            xim_preedit_font: raw.xim_preedit_font,
            hangul_engine: HangulEngine::new(
                &raw.hangul,
                kime_engine_backend_hangul::builtin_layouts(),
            ),
            latin_engine: LatinEngine::new(&raw.latin),
        }
    }

    #[cfg(unix)]
    pub fn from_raw_config_with_dir(raw: RawConfig, dir: &xdg::BaseDirectories) -> Self {
        Self {
            default_category: raw.default_category,
            global_category_state: raw.global_category_state,
            hotkeys: raw.hotkeys.into_iter().collect(),
            icon_color: raw.icon_color,
            xim_preedit_font: raw.xim_preedit_font,
            hangul_engine: HangulEngine::from_config_with_dir(&raw.hangul, dir),
            latin_engine: LatinEngine::new(&raw.latin),
        }
    }

    #[cfg(unix)]
    pub fn load_from_config_dir() -> Option<Self> {
        let dir = xdg::BaseDirectories::with_prefix("kime").ok()?;
        let raw = dir
            .find_config_file("config.yaml")
            .and_then(|config| serde_yaml::from_reader(std::fs::File::open(config).ok()?).ok())
            .unwrap_or_default();

        Some(Self::from_raw_config_with_dir(raw, &dir))
    }
}
