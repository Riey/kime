use crate::{keycode::Key, KeyCode, Layout, ModifierState};
use ahash::AHashMap;
use enumset::{EnumSet, EnumSetType};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Hash, Serialize, Deserialize, EnumSetType)]
#[enumset(serialize_as_list)]
pub enum Addon {
    ComposeChoseongSsang,
    ComposeJungseongSsang,
    ComposeJongseongSsang,
    DecomposeChoseongSsang,
    DecomposeJungseongSsang,
    DecomposeJongseongSsang,

    /// ㅏ + ㄱ = 가
    FlexibleComposeOrder,

    /// 안 + ㅣ = 아니
    TreatJongseongAsChoseong,
    /// 읅 + ㄱ = 을ㄲ
    TreatJongseongAsChoseongCompose,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub enum HotkeyBehavior {
    ToggleHangul,
    ToHangul,
    ToEnglish,
    Commit,
    Emoji,
    Hanja,
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
    pub const fn new(behavior: HotkeyBehavior, result: HotkeyResult) -> Self {
        Self { behavior, result }
    }

    pub const fn behavior(self) -> HotkeyBehavior {
        self.behavior
    }
    pub const fn result(self) -> HotkeyResult {
        self.result
    }
}

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct RawConfig {
    pub layout: String,
    pub global_hangul_state: bool,
    pub word_commit: bool,
    pub hotkeys: BTreeMap<Key, Hotkey>,
    pub layout_addons: BTreeMap<String, EnumSet<Addon>>,
    pub xim_preedit_font: (String, f64),
}

impl Default for RawConfig {
    fn default() -> Self {
        Self {
            layout: "dubeolsik".to_string(),
            global_hangul_state: false,
            word_commit: false,
            hotkeys: [
                (
                    Key::normal(KeyCode::Esc),
                    Hotkey::new(HotkeyBehavior::ToEnglish, HotkeyResult::Bypass),
                ),
                (
                    Key::normal(KeyCode::AltR),
                    Hotkey::new(HotkeyBehavior::ToggleHangul, HotkeyResult::Consume),
                ),
                (
                    Key::normal(KeyCode::Muhenkan),
                    Hotkey::new(HotkeyBehavior::ToggleHangul, HotkeyResult::Consume),
                ),
                (
                    Key::normal(KeyCode::Hangul),
                    Hotkey::new(HotkeyBehavior::ToggleHangul, HotkeyResult::Consume),
                ),
                (
                    Key::super_(KeyCode::Space),
                    Hotkey::new(HotkeyBehavior::ToggleHangul, HotkeyResult::Consume),
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
            layout_addons: vec![
                ("all".into(), EnumSet::only(Addon::ComposeChoseongSsang)),
                (
                    "dubeolsik".into(),
                    EnumSet::only(Addon::TreatJongseongAsChoseong),
                ),
            ]
            .into_iter()
            .collect(),
            xim_preedit_font: ("D2Coding".to_string(), 15.0),
        }
    }
}

pub struct Config {
    pub layout: Layout,
    pub global_hangul_state: bool,
    pub hotkeys: AHashMap<Key, Hotkey>,
    pub layout_addons: EnumSet<Addon>,
    pub word_commit: bool,
    pub xim_preedit_font: (String, f64),
}

impl Default for Config {
    fn default() -> Self {
        Self::from_raw_config(RawConfig::default())
    }
}

pub const BUILTIN_LAYOUTS: &'static [(&'static str, &'static str)] = &[
    ("dubeolsik", include_str!("../data/dubeolsik.yaml")),
    (
        "sebeolsik-3-90",
        include_str!("../data/sebeolsik-3-90.yaml"),
    ),
    (
        "sebeolsik-3-91",
        include_str!("../data/sebeolsik-3-91.yaml"),
    ),
    (
        "sebeolsik-3-2015",
        include_str!("../data/sebeolsik-3-2015.yaml"),
    ),
    (
        "sebeolsik-3sin-1995",
        include_str!("../data/sebeolsik-3sin-1995.yaml"),
    ),
];

impl Config {
    pub fn new(layout: Layout, raw: RawConfig) -> Self {
        Self {
            layout,
            global_hangul_state: raw.global_hangul_state,
            word_commit: raw.word_commit,
            layout_addons: raw
                .layout_addons
                .get("all")
                .copied()
                .unwrap_or_default()
                .union(
                    raw.layout_addons
                        .get(&raw.layout)
                        .copied()
                        .unwrap_or_default(),
                ),
            hotkeys: raw.hotkeys.into_iter().collect(),
            xim_preedit_font: raw.xim_preedit_font,
        }
    }

    pub fn from_raw_config(raw: RawConfig) -> Self {
        let layout = BUILTIN_LAYOUTS
            .iter()
            .copied()
            .find_map(|(name, layout)| {
                if name == raw.layout {
                    Layout::load_from(layout).ok()
                } else {
                    None
                }
            })
            .unwrap_or_default();

        Self::new(layout, raw)
    }

    #[cfg(unix)]
    pub fn from_raw_config_with_dir(raw: RawConfig, dir: xdg::BaseDirectories) -> Self {
        if let Some(layout) = dir
            .list_config_files("layouts")
            .into_iter()
            .find_map(|layout| {
                if layout.file_stem()?.to_str()? == raw.layout {
                    Some(Layout::from_items(
                        serde_yaml::from_reader(std::fs::File::open(layout).ok()?).ok()?,
                    ))
                } else {
                    None
                }
            })
        {
            Self::new(layout, raw)
        } else {
            Self::from_raw_config(raw)
        }
    }

    #[cfg(unix)]
    pub fn load_from_config_dir() -> Option<Self> {
        let dir = xdg::BaseDirectories::with_prefix("kime").ok()?;

        let raw = dir
            .find_config_file("config.yaml")
            .and_then(|config| serde_yaml::from_reader(std::fs::File::open(config).ok()?).ok())
            .unwrap_or_default();

        Some(Self::from_raw_config_with_dir(raw, dir))
    }

    pub fn word_commit(&self) -> bool {
        self.word_commit
    }

    pub fn check_addon(&self, addon: Addon) -> bool {
        self.layout_addons.contains(addon)
    }
}
