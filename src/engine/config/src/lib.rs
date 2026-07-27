use enum_map::Enum;
use enumset::{enum_set, EnumSetType};
use maplit::btreemap;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

pub use kime_engine_backend::{Key, KeyCode, ModifierState};
pub use kime_engine_backend_hangul::{HangulConfig, HangulData};
pub use kime_engine_backend_latin::{LatinConfig, LatinData};

pub use enum_map::{enum_map, EnumMap};
pub use enumset::EnumSet;
pub use log::LevelFilter;

#[derive(Debug, EnumSetType, Enum, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", enumset(serialize_repr = "list"))]
#[repr(u32)]
pub enum InputCategory {
    Latin,
    Hangul,
}

#[derive(Debug, EnumSetType, Enum, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", enumset(serialize_repr = "list"))]
#[repr(u32)]
pub enum InputMode {
    Math,
    Hanja,
    Emoji,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Copy, Debug)]
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

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Copy, Debug)]
pub enum HotkeyResult {
    Consume,
    Bypass,
    ConsumeIfProcessed,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Copy, Debug)]
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

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Copy)]
#[repr(C)]
pub enum IconColor {
    White,
    Black,
}

impl Default for IconColor {
    fn default() -> Self {
        Self::Black
    }
}

#[derive(EnumSetType)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", enumset(serialize_repr = "list"))]
pub enum DaemonModule {
    Xim,
    Wayland,
    Indicator,
}

#[derive(Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct IndicatorConfig {
    pub icon_color: IconColor,
}

#[derive(Clone, Copy)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct DaemonConfig {
    pub modules: EnumSet<DaemonModule>,
}

impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            modules: enum_set![DaemonModule::Xim | DaemonModule::Wayland | DaemonModule::Indicator],
        }
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct LogConfig {
    pub global_level: log::LevelFilter,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            global_level: log::LevelFilter::Debug,
        }
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct EngineConfig {
    pub translation_layer: Option<PathBuf>,
    pub default_category: InputCategory,
    pub global_category_state: bool,
    pub global_hotkeys: BTreeMap<Key, Hotkey>,
    pub category_hotkeys: BTreeMap<InputCategory, BTreeMap<Key, Hotkey>>,
    pub mode_hotkeys: BTreeMap<InputMode, BTreeMap<Key, Hotkey>>,
    pub candidate_font: String,
    pub xim_preedit_font: (String, f32),
    pub latin: LatinConfig,
    pub hangul: HangulConfig,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            translation_layer: None,
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
            xim_preedit_font: ("Noto Sans CJK KR".to_string(), 15.0),
            candidate_font: "Noto Sans CJK KR".to_string(),
        }
    }
}

#[derive(Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct RawConfig {
    pub daemon: DaemonConfig,
    pub indicator: IndicatorConfig,
    pub log: LogConfig,
    pub engine: EngineConfig,
}
