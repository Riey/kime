use enum_map::{Enum, EnumMap};
use enumset::EnumSetType;
use kime_engine_backend::{Key, KeyCode, KeyMap, ModifierState};
use kime_engine_backend_hangul::{HangulConfig, HangulData};
use kime_engine_backend_latin::{LatinConfig, LatinData};
use maplit::btreemap;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::time::SystemTime;

pub type FontString = arraystring::ArrayString<arraystring::typenum::U31>;

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

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct RawConfig {
    pub default_category: InputCategory,
    pub global_category_state: bool,
    pub icon_color: IconColor,
    pub global_hotkeys: BTreeMap<Key, Hotkey>,
    pub category_hotkeys: BTreeMap<InputCategory, BTreeMap<Key, Hotkey>>,
    pub mode_hotkeys: BTreeMap<InputMode, BTreeMap<Key, Hotkey>>,
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
            xim_preedit_font: ("D2Coding".into(), 15.0),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Config {
    pub is_shm: bool,
    pub timestamp: Option<SystemTime>,
    pub abi_version: u32,
    pub global_category_state: bool,
    pub default_category: InputCategory,
    pub category_hotkeys: EnumMap<InputCategory, KeyMap<Hotkey>>,
    pub mode_hotkeys: EnumMap<InputMode, KeyMap<Hotkey>>,
    pub icon_color: IconColor,
    pub xim_preedit_font: (FontString, f64),
    pub hangul_data: HangulData,
    pub latin_data: LatinData,
}

impl Default for Config {
    fn default() -> Self {
        Self::new(RawConfig::default())
    }
}

impl Config {
    pub const ABI_VERSION: u32 = 1;

    fn new_impl(mut raw: RawConfig, hangul_data: HangulData) -> Self {
        Self {
            is_shm: false,
            timestamp: None,
            abi_version: Self::ABI_VERSION,
            default_category: raw.default_category,
            global_category_state: raw.global_category_state,
            category_hotkeys: EnumMap::from(|cat| {
                if let Some(map) = raw.category_hotkeys.get_mut(&cat) {
                    raw.global_hotkeys
                        .iter()
                        .chain(map.iter())
                        .map(|(k, v)| (*k, *v))
                        .collect()
                } else {
                    raw.global_hotkeys.iter().map(|(k, v)| (*k, *v)).collect()
                }
            }),
            mode_hotkeys: EnumMap::from(|mode| {
                if let Some(map) = raw.mode_hotkeys.get_mut(&mode) {
                    raw.global_hotkeys
                        .iter()
                        .chain(map.iter())
                        .map(|(k, v)| (*k, *v))
                        .collect()
                } else {
                    raw.global_hotkeys.iter().map(|(k, v)| (*k, *v)).collect()
                }
            }),
            icon_color: raw.icon_color,
            xim_preedit_font: (
                FontString::from_str_truncate(raw.xim_preedit_font.0),
                raw.xim_preedit_font.1,
            ),
            latin_data: LatinData::new(&raw.latin),
            hangul_data,
        }
    }

    pub fn new(raw: RawConfig) -> Self {
        let hangul_data =
            HangulData::new(&raw.hangul, kime_engine_backend_hangul::builtin_layouts());

        Self::new_impl(raw, hangul_data)
    }

    #[cfg(unix)]
    pub fn from_raw_config_with_dir(raw: RawConfig, dir: &xdg::BaseDirectories) -> Self {
        let hangul_data = HangulData::from_config_with_dir(&raw.hangul, dir);
        Self::new_impl(raw, hangul_data)
    }

    #[cfg(unix)]
    pub fn config_file_timestamp() -> Option<SystemTime> {
        let dir = xdg::BaseDirectories::with_prefix("kime").ok()?;
        let path = dir.find_config_file("config.yaml")?;
        std::fs::metadata(path).ok()?.accessed().ok()
    }

    #[cfg(unix)]
    pub fn load_from_config_dir() -> Option<Self> {
        let dir = xdg::BaseDirectories::with_prefix("kime").ok()?;
        dir.find_config_file("config.yaml").and_then(|config| {
            let file = std::fs::File::open(config).ok()?;
            let time = file.metadata().and_then(|m| m.accessed()).ok();
            let raw = serde_yaml::from_reader(file).unwrap_or_default();
            let mut ret = Self::from_raw_config_with_dir(raw, &dir);
            ret.timestamp = time;
            Some(ret)
        })
    }
}
