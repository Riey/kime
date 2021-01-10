use crate::Layout;
use ahash::AHashSet;
use serde::{Deserialize, Serialize};
use xkbcommon::xkb;

#[derive(Serialize, Deserialize)]
struct RawConfig {
    layout: String,
    esc_turn_off: bool,
    hangul_symbols: Vec<String>,
    xim_preedit_font: String,
}

impl Default for RawConfig {
    fn default() -> Self {
        Self {
            layout: "dubeolsik".to_string(),
            esc_turn_off: true,
            hangul_symbols: vec![
                "Hangul".to_string(),
                "Henkan".to_string(),
                "Alt_R".to_string(),
            ],
            xim_preedit_font: "D2Coding".to_string(),
        }
    }
}

pub struct Config {
    pub(crate) layout: Layout,
    pub(crate) esc_turn_off: bool,
    pub(crate) hangul_symbols: AHashSet<xkb::Keysym>,
    pub xim_preedit_font: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            layout: Layout::default(),
            esc_turn_off: true,
            hangul_symbols: [xkb::KEY_Hangul, xkb::KEY_Henkan, xkb::KEY_Alt_R]
                .iter()
                .copied()
                .collect(),
            xim_preedit_font: "D2Coding".to_string(),
        }
    }
}

impl Config {
    pub fn new(
        layout: Layout,
        esc_turn_off: bool,
        hangul_symbols: AHashSet<xkb::Keysym>,
        xim_preedit_font: String,
    ) -> Self {
        Self {
            layout,
            esc_turn_off,
            hangul_symbols,
            xim_preedit_font,
        }
    }

    pub fn load_from_config_dir() -> Option<Self> {
        let dir = xdg::BaseDirectories::with_prefix("kime").ok()?;

        let config = match dir.find_config_file("config.yaml") {
            Some(config) => config,
            None => {
                let path  = dir.place_config_file("config.yaml").ok()?;
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
            hangul_symbols: config
                .hangul_symbols
                .iter()
                .filter_map(|s| match xkb::keysym_from_name(s, 0) {
                    xkb::KEY_NoSymbol => None,
                    s => Some(s),
                })
                .collect(),
            xim_preedit_font: config.xim_preedit_font,
            esc_turn_off: config.esc_turn_off,
        })
    }
}
