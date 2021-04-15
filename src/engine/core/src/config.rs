pub use kime_engine_config::*;

/// Preprocessed engine config
pub struct Config {
    pub default_category: InputCategory,
    pub global_category_state: bool,
    pub category_hotkeys: EnumMap<InputCategory, Vec<(Key, Hotkey)>>,
    pub mode_hotkeys: EnumMap<InputMode, Vec<(Key, Hotkey)>>,
    pub xim_preedit_font: (String, f64),
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
        Self {
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
            xim_preedit_font: engine.xim_preedit_font,
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
pub fn load_from_config_dir() -> Option<(Config, DaemonConfig, IndicatorConfig)> {
    let dir = xdg::BaseDirectories::with_prefix("kime").ok()?;
    let config: RawConfig = dir
        .find_config_file("config.yaml")
        .and_then(|config| serde_yaml::from_reader(std::fs::File::open(config).ok()?).ok())
        .unwrap_or_default();

    Some((
        Config::from_engine_config_with_dir(config.engine, &dir),
        config.daemon,
        config.indicator,
    ))
}
