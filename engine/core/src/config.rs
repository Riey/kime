use crate::{keycode::Key, KeyCode, Layout};
use ahash::AHashSet;
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

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct RawConfig {
    pub layout: String,
    pub esc_turn_off: bool,
    pub hangul_keys: Vec<String>,
    pub xim_preedit_font: (String, f64),
    pub gtk_commit_english: bool,
    pub compose: ComposeConfig,
}

impl Default for RawConfig {
    fn default() -> Self {
        const DEFAULT_HANGUL_KEYS: &[Key] = &[
            Key::normal(KeyCode::AltR),
            Key::normal(KeyCode::Henkan),
            Key::normal(KeyCode::Hangul),
            Key::super_(KeyCode::Space),
        ];

        Self {
            layout: "dubeolsik".to_string(),
            esc_turn_off: true,
            hangul_keys: DEFAULT_HANGUL_KEYS
                .iter()
                .map(ToString::to_string)
                .collect(),
            xim_preedit_font: ("D2Coding".to_string(), 15.0),
            gtk_commit_english: true,
            compose: ComposeConfig::default(),
        }
    }
}

pub struct Config {
    pub(crate) layout: Layout,
    pub(crate) esc_turn_off: bool,
    pub(crate) hangul_keys: AHashSet<Key>,
    pub(crate) compose: ComposeConfig,
    pub xim_preedit_font: (String, f64),
    pub gtk_commit_english: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self::new(Layout::default(), RawConfig::default())
    }
}

impl Config {
    pub fn new(layout: Layout, raw: RawConfig) -> Self {
        Self {
            layout,
            esc_turn_off: raw.esc_turn_off,
            compose: raw.compose,
            hangul_keys: raw
                .hangul_keys
                .iter()
                .filter_map(|s| s.parse().ok())
                .collect(),
            xim_preedit_font: raw.xim_preedit_font,
            gtk_commit_english: raw.gtk_commit_english,
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
