use crate::{keycode::Key, KeyCode, Layout};
use ahash::AHashMap;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct ComposeConfig {
    pub compose_choseong_ssang: bool,
    pub decompose_choseong_ssang: bool,
    pub compose_jungseong_ssang: bool,
    pub decompose_jungseong_ssang: bool,
    pub compose_jongseong_ssang: bool,
    pub decompose_jongseong_ssang: bool,
}

impl Default for ComposeConfig {
    fn default() -> Self {
        Self {
            compose_choseong_ssang: true,
            decompose_choseong_ssang: false,
            compose_jungseong_ssang: false,
            decompose_jungseong_ssang: false,
            compose_jongseong_ssang: false,
            decompose_jongseong_ssang: false,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub enum HotkeyBehavior {
    ToggleHangul,
    ToHangul,
    ToEnglish,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub enum HotkeyResult {
    Consume,
    Bypass,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
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
    pub hotkeys: AHashMap<Key, Hotkey>,
    pub xim_preedit_font: (String, f64),
    pub compose: ComposeConfig,
}

impl Default for RawConfig {
    fn default() -> Self {
        Self {
            layout: "dubeolsik".to_string(),
            global_hangul_state: false,
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
            ]
            .iter()
            .copied()
            .collect(),
            xim_preedit_font: ("D2Coding".to_string(), 15.0),
            compose: ComposeConfig::default(),
        }
    }
}

pub struct Config {
    pub(crate) layout: Layout,
    pub(crate) global_hangul_state: bool,
    pub(crate) hotkeys: AHashMap<Key, Hotkey>,
    pub(crate) compose: ComposeConfig,
    pub xim_preedit_font: (String, f64),
}

impl Default for Config {
    fn default() -> Self {
        Self::from_raw_config(RawConfig::default(), None)
    }
}

impl Config {
    pub fn new(layout: Layout, raw: RawConfig) -> Self {
        Self {
            layout,
            global_hangul_state: raw.global_hangul_state,
            compose: raw.compose,
            hotkeys: raw.hotkeys,
            xim_preedit_font: raw.xim_preedit_font,
        }
    }

    pub fn from_raw_config(raw: RawConfig, dir: Option<xdg::BaseDirectories>) -> Self {
        let layout = dir
            .and_then(|dir| {
                dir.list_config_files("layouts")
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
            })
            .unwrap_or_else(|| {
                // User layout not exists fallback to embeded layouts
                match raw.layout.as_str() {
                    "dubeolsik" => Layout::load_from(include_str!("../data/dubeolsik.yaml"))
                        .expect("Load dubeolsik layout"),
                    "sebeolsik-390" => {
                        Layout::load_from(include_str!("../data/sebeolsik-390.yaml"))
                            .expect("Load sebeolsik-390 layout")
                    }
                    "sebeolsik-391" => {
                        Layout::load_from(include_str!("../data/sebeolsik-391.yaml"))
                            .expect("Load sebeolsik-391 layout")
                    }
                    // custom layout
                    other => {
                        eprintln!("Can't find layout {}", other);
                        Layout::default()
                    }
                }
            });

        Self::new(layout, raw)
    }

    pub fn load_from_config_dir() -> Option<Self> {
        let dir = xdg::BaseDirectories::with_prefix("kime").ok()?;

        let raw = dir
            .find_config_file("config.yaml")
            .and_then(|config| serde_yaml::from_reader(std::fs::File::open(config).ok()?).ok())
            .unwrap_or_default();

        Some(Self::from_raw_config(raw, Some(dir)))
    }
}
