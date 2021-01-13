use crate::{
    keycode::{Key, KeyCode},
    Layout,
};
use ahash::AHashSet;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(default)]
struct RawConfig {
    layout: String,
    esc_turn_off: bool,
    hangul_keys: Vec<String>,
    xim_preedit_font: String,
    gtk_commit_english: bool,
}

impl Default for RawConfig {
    fn default() -> Self {
        Self {
            layout: "dubeolsik".to_string(),
            esc_turn_off: true,
            hangul_keys: vec![
                "Hangul".to_string(),
                "Henkan".to_string(),
                "Alt_R".to_string(),
            ],
            xim_preedit_font: "D2Coding".to_string(),
            gtk_commit_english: true,
        }
    }
}

pub struct Config {
    pub(crate) layout: Layout,
    pub(crate) esc_turn_off: bool,
    pub(crate) hangul_keys: AHashSet<Key>,
    pub xim_preedit_font: String,
    pub gtk_commit_english: bool,
}

impl Default for Config {
    fn default() -> Self {
        let mut hangul_keys = AHashSet::new();
        for key in [KeyCode::Hangul, KeyCode::Henkan, KeyCode::AltR].iter() {
            hangul_keys.insert(Key::new(*key, false));
        }

        Self {
            layout: Layout::default(),
            esc_turn_off: true,
            hangul_keys,
            xim_preedit_font: "D2Coding".to_string(),
            gtk_commit_english: true,
        }
    }
}

impl Config {
    pub fn new(
        layout: Layout,
        esc_turn_off: bool,
        hangul_keys: AHashSet<Key>,
        xim_preedit_font: String,
        gtk_commit_english: bool,
    ) -> Self {
        Self {
            layout,
            esc_turn_off,
            hangul_keys,
            xim_preedit_font,
            gtk_commit_english,
        }
    }

    pub fn load_from_config_dir() -> Option<Self> {
        let dir = xdg::BaseDirectories::with_prefix("kime").ok()?;

        let config = match dir.find_config_file("config.yaml") {
            Some(config) => config,
            None => {
                let path = dir.place_config_file("config.yaml").ok()?;
                std::fs::write(&path, serde_yaml::to_string(&RawConfig::default()).ok()?).ok()?;
                path
            }
        };

        let config: RawConfig =
            serde_yaml::from_reader(std::fs::File::open(config).ok()?).unwrap_or_default();

        let layout = dir
            .list_data_files("layouts")
            .into_iter()
            .find_map(|layout| {
                if layout.file_stem()?.to_str()? == config.layout {
                    Some(Layout::from_items(
                        serde_yaml::from_reader(std::fs::File::open(layout).ok()?).ok()?,
                    ))
                } else {
                    None
                }
            })
            .unwrap_or_default();

        Some(Self {
            layout,
            hangul_keys: config
                .hangul_keys
                .iter()
                .filter_map(|s| s.parse().ok())
                .collect(),
            xim_preedit_font: config.xim_preedit_font,
            esc_turn_off: config.esc_turn_off,
            gtk_commit_english: config.gtk_commit_english,
        })
    }
}
