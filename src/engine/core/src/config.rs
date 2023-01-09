use crate::KeyMap;
use fontdb::{Family, Query};
pub use kime_engine_config::*;
use std::{borrow::Cow, fs};

/// Preprocessed engine config
pub struct Config {
    pub translation_layer: Option<KeyMap<Key>>,
    pub default_category: InputCategory,
    pub global_category_state: bool,
    pub category_hotkeys: EnumMap<InputCategory, Vec<(Key, Hotkey)>>,
    pub mode_hotkeys: EnumMap<InputMode, Vec<(Key, Hotkey)>>,
    pub candidate_font: (Cow<'static, [u8]>, u32),
    pub xim_preedit_font: (Cow<'static, [u8]>, u32, f32),
    pub hangul_data: HangulData,
    pub preferred_direct: bool,
    pub latin_data: LatinData,
}

impl Default for Config {
    fn default() -> Self {
        Self::new(EngineConfig::default())
    }
}

impl Config {
    fn new_impl(mut engine: EngineConfig, hangul_data: HangulData) -> Self {
        let mut db = fontdb::Database::new();
        db.load_system_fonts();

        let load_font = |name| {
            db.query(&Query {
                families: &[Family::Name(name), Family::Serif],
                ..Default::default()
            })
            .and_then(|id| db.with_face_data(id, |data, index| (Cow::Owned(data.to_vec()), index)))
            .unwrap_or((
                Cow::Borrowed(include_bytes!("../fonts/D2Coding-Ver1.3.2-20180524.ttf").as_slice()),
                0,
            ))
        };

        let translation_layer: Option<KeyMap<Key>> = engine
            .translation_layer
            .and_then(|f| {
                xdg::BaseDirectories::with_prefix("kime")
                    .ok()
                    .and_then(|d| d.find_config_file(f))
            })
            .as_ref()
            .and_then(|f| fs::read_to_string(f.as_path()).ok())
            .as_ref()
            .and_then(|content| serde_yaml::from_str(content).ok());

        Self {
            translation_layer,
            default_category: engine.default_category,
            global_category_state: engine.global_category_state,
            category_hotkeys: enum_map! {
                cat => {
                    if let Some(map) = engine.category_hotkeys.get_mut(&cat) { for (k, v) in engine.global_hotkeys.iter() {
                            map.entry(*k).or_insert(*v);
                        }
                        map.iter().map(|(k, v)| (*k, *v)).collect()
                    } else {
                        engine.global_hotkeys.iter().map(|(k, v)| (*k, *v)).collect()
                    }
                }
            },
            mode_hotkeys: enum_map! {
                mode => {
                    if let Some(map) = engine.mode_hotkeys.get_mut(&mode) {
                        for (k, v) in engine.global_hotkeys.iter() {
                            map.entry(*k).or_insert(*v);
                        }
                        map.iter().map(|(k, v)| (*k, *v)).collect()
                    } else {
                        engine.global_hotkeys.iter().map(|(k, v)| (*k, *v)).collect()
                    }
                }
            },
            xim_preedit_font: {
                let (font, index) = load_font(&engine.xim_preedit_font.0);
                (font, index, engine.xim_preedit_font.1)
            },
            candidate_font: {
                let (font, index) = load_font(&engine.candidate_font);
                (font, index)
            },
            preferred_direct: engine.latin.preferred_direct,
            latin_data: LatinData::new(&engine.latin),
            hangul_data,
        }
    }

    pub fn new(engine: EngineConfig) -> Self {
        let hangul_data = HangulData::new(
            &engine.hangul,
            kime_engine_backend_hangul::builtin_layouts(),
        );

        Self::new_impl(engine, hangul_data)
    }

    #[cfg(unix)]
    pub fn from_engine_config_with_dir(engine: EngineConfig, dir: &xdg::BaseDirectories) -> Self {
        let hangul_data = HangulData::from_config_with_dir(&engine.hangul, dir);
        Self::new_impl(engine, hangul_data)
    }
}

#[cfg(unix)]
pub fn load_engine_config_from_config_dir() -> Option<Config> {
    let dir = xdg::BaseDirectories::with_prefix("kime").ok()?;
    let config: RawConfig = dir
        .find_config_file("config.yaml")
        .and_then(|config| serde_yaml::from_reader(std::fs::File::open(config).ok()?).ok())
        .unwrap_or_default();

    Some(Config::from_engine_config_with_dir(config.engine, &dir))
}

#[cfg(unix)]
pub fn load_other_configs_from_config_dir() -> Option<(DaemonConfig, IndicatorConfig, LogConfig)> {
    let dir = xdg::BaseDirectories::with_prefix("kime").ok()?;
    let config: RawConfig = dir
        .find_config_file("config.yaml")
        .and_then(|config| serde_yaml::from_reader(std::fs::File::open(config).ok()?).ok())
        .unwrap_or_default();

    Some((config.daemon, config.indicator, config.log))
}
