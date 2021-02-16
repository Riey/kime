use crate::{keycode::Key, KeyCode, Layout};
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
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub enum HotkeyBehavior {
    ToggleHangul,
    ToHangul,
    ToEnglish,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub enum HotkeyResult {
    Consume,
    Bypass,
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
    pub hotkeys: BTreeMap<Key, Hotkey>,
    pub layout_addons: BTreeMap<String, EnumSet<Addon>>,
    pub xim_preedit_font: (String, f64),
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
            layout_addons: std::iter::once((
                "all".into(),
                EnumSet::only(Addon::ComposeChoseongSsang),
            ))
            .collect(),
            xim_preedit_font: ("D2Coding".to_string(), 15.0),
        }
    }
}

pub struct Config {
    pub(crate) layout: Layout,
    pub(crate) global_hangul_state: bool,
    pub(crate) hotkeys: AHashMap<Key, Hotkey>,
    layout_addons: EnumSet<Addon>,
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

    pub fn from_raw_config(raw: RawConfig, dir: Option<xdg::BaseDirectories>) -> Self {
        let layout = dir
            .and_then(|dir| {
                dir.list_config_files("layouts")
                    .into_iter()
                    .find_map(|layout| {
                        log::info!("Try Read custom layout: {}...", layout.display());
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
                macro_rules! load_builtin_layout {
                    ($($name:expr),+) => {
                        match raw.layout.as_str() {
                            $(
                                $name => Layout::load_from(include_str!(concat!(concat!("../data/", $name), ".yaml"))).unwrap_or_else(|_| {
                                    log::error!("Can't load builtin layout {} fallback to empty layout", $name);
                                    Layout::default()
                                }),
                            )+
                            other => {
                                log::error!("Can't find layout {} fallback to empty layout", other);
                                Layout::default()
                            }
                        }
                    }
                }

                load_builtin_layout!("dubeolsik", "sebeolsik-390", "sebeolsik-391", "sebeolsik-sin1995")
            });

        Self::new(layout, raw)
    }

    pub fn load_from_config_dir() -> Option<Self> {
        let dir = xdg::BaseDirectories::with_prefix("kime").ok()?;

        let raw = dir
            .find_config_file("config.yaml")
            .and_then(|config| {
                log::info!("Found config file: {}", config.display());
                match serde_yaml::from_reader(std::fs::File::open(config).ok()?) {
                    Ok(config) => Some(config),
                    Err(err) => {
                        log::error!("Read config error: {}", err);
                        None
                    }
                }
            })
            .unwrap_or_default();

        Some(Self::from_raw_config(raw, Some(dir)))
    }

    pub fn check_addon(&self, addon: Addon) -> bool {
        self.layout_addons.contains(addon)
    }
}
