use enum_map::Enum;
use enumset::EnumSetType;
use maplit::btreemap;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub use kime_engine_backend::{Key, KeyCode, ModifierState};
pub use kime_engine_backend_hangul::HangulConfig;
pub use kime_engine_backend_latin::LatinConfig;

pub use enum_map::{enum_map, EnumMap};
pub use enumset::EnumSet;

#[derive(Serialize, Deserialize, Debug, EnumSetType, Enum, PartialOrd, Ord)]
#[enumset(serialize_as_list)]
#[repr(u32)]
pub enum InputCategory {
    Latin,
    Hangul,
}

#[derive(Serialize, Deserialize, Debug, EnumSetType, Enum, PartialOrd, Ord)]
#[enumset(serialize_as_list)]
#[repr(u32)]
pub enum InputMode {
    Math,
    Hanja,
    Emoji,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub enum HotkeyBehavior {
    Switch(InputCategory),
    Toggle(InputCategory, InputCategory),
    Mode(InputMode),
    Commit,
    Ignore,
}

impl HotkeyBehavior {
    pub const fn toggle_hangul_latin() -> Self {
        Self::Toggle(InputCategory::Hangul, InputCategory::Latin)
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
    result: HotkeyResult,
}

impl Hotkey {
    pub fn new(behavior: HotkeyBehavior, result: HotkeyResult) -> Self {
        Self { behavior, result }
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

#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum Module {
    Xim,
    Wayland,
    Indicator,
}

#[derive(Serialize, Deserialize, Default)]
pub struct IndicatorConfig {
    pub icon_color: IconColor,
}

#[derive(Serialize, Deserialize)]
pub struct DaemonConfig {
    pub modules: Vec<Module>,
}

impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            modules: vec![Module::Xim, Module::Wayland, Module::Indicator],
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct EngineConfig {
    pub default_category: InputCategory,
    pub global_category_state: bool,
    pub global_hotkeys: BTreeMap<Key, Hotkey>,
    pub category_hotkeys: BTreeMap<InputCategory, BTreeMap<Key, Hotkey>>,
    pub mode_hotkeys: BTreeMap<InputMode, BTreeMap<Key, Hotkey>>,
    pub xim_preedit_font: (String, f64),
    pub latin: LatinConfig,
    pub hangul: HangulConfig,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            latin: LatinConfig::default(),
            hangul: HangulConfig::default(),
            default_category: InputCategory::Latin,
            global_category_state: false,
            global_hotkeys: btreemap! {
                Key::normal(KeyCode::Esc) => Hotkey::new(HotkeyBehavior::Switch(InputCategory::Latin), HotkeyResult::Bypass),
                Key::normal(KeyCode::AltR) => Hotkey::new(HotkeyBehavior::toggle_hangul_latin(), HotkeyResult::Consume),
                Key::normal(KeyCode::Hangul) => Hotkey::new(HotkeyBehavior::toggle_hangul_latin(), HotkeyResult::Consume),
                Key::super_(KeyCode::Space) => Hotkey::new(HotkeyBehavior::toggle_hangul_latin(), HotkeyResult::Consume),
                Key::normal(KeyCode::Muhenkan) => Hotkey::new(HotkeyBehavior::toggle_hangul_latin(), HotkeyResult::Consume),
                Key::new(KeyCode::E, ModifierState::CONTROL | ModifierState::ALT) => Hotkey::new(HotkeyBehavior::Mode(InputMode::Emoji), HotkeyResult::ConsumeIfProcessed),
                Key::new(KeyCode::Backslash, ModifierState::CONTROL | ModifierState::ALT) => Hotkey::new(HotkeyBehavior::Mode(InputMode::Math), HotkeyResult::ConsumeIfProcessed),
            },
            category_hotkeys: btreemap! {
                InputCategory::Hangul => btreemap! {
                    Key::normal(KeyCode::F9) => Hotkey::new(HotkeyBehavior::Mode(InputMode::Hanja), HotkeyResult::ConsumeIfProcessed),
                    Key::normal(KeyCode::HangulHanja) => Hotkey::new(HotkeyBehavior::Mode(InputMode::Hanja), HotkeyResult::Consume),
                    Key::normal(KeyCode::ControlR) => Hotkey::new(HotkeyBehavior::Mode(InputMode::Hanja), HotkeyResult::Consume),
                },
            },
            mode_hotkeys: btreemap! {
                InputMode::Hanja => btreemap! {
                    Key::normal(KeyCode::Enter) => Hotkey::new(HotkeyBehavior::Commit, HotkeyResult::ConsumeIfProcessed),
                    Key::normal(KeyCode::Tab) => Hotkey::new(HotkeyBehavior::Commit, HotkeyResult::ConsumeIfProcessed),
                },
                InputMode::Emoji => btreemap! {
                    Key::normal(KeyCode::Enter) => Hotkey::new(HotkeyBehavior::Commit, HotkeyResult::ConsumeIfProcessed),
                    Key::normal(KeyCode::Tab) => Hotkey::new(HotkeyBehavior::Commit, HotkeyResult::ConsumeIfProcessed),
                },
                InputMode::Math => btreemap! {
                    Key::normal(KeyCode::Enter) => Hotkey::new(HotkeyBehavior::Commit, HotkeyResult::ConsumeIfProcessed),
                    Key::normal(KeyCode::Tab) => Hotkey::new(HotkeyBehavior::Commit, HotkeyResult::ConsumeIfProcessed),
                },
            },
            xim_preedit_font: ("D2Coding".to_string(), 15.0),
        }
    }
}

#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct RawConfig {
    pub daemon: DaemonConfig,
    pub indicator: IndicatorConfig,
    pub engine: EngineConfig,
}
