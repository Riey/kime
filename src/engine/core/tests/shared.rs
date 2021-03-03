use kime_engine_core::{Config, InputCategory, InputEngine, InputResult, Key};

#[track_caller]
pub fn test_input_impl(config: &Config, keys: &[(Key, &str, &str)]) {
    let mut engine = InputEngine::new(config);

    engine.set_input_category(config, InputCategory::Hangul);

    for (key, preedit, commit) in keys.iter().copied() {
        eprintln!("Key: {:?}", key);

        let ret = engine.press_key(key, &config);

        eprintln!("Ret: {:?}", ret);

        if ret.contains(InputResult::HAS_PREEDIT) {
            assert_eq!(preedit, engine.preedit_str());
        } else {
            assert!(preedit.is_empty());
        }

        if !ret.contains(InputResult::CONSUMED) {
            assert_eq!(commit, format!("{}PASS", engine.commit_str()));
        } else if ret.intersects(InputResult::NEED_RESET | InputResult::NEED_FLUSH) {
            assert_eq!(commit, engine.commit_str());
        } else {
            assert!(commit.is_empty());
        }

        if ret.contains(InputResult::NEED_RESET) {
            engine.reset();
        } else if ret.contains(InputResult::NEED_FLUSH) {
            engine.flush();
        }
    }
}

#[allow(unused_macros)]
macro_rules! define_layout_test {
    ($layout:literal) => {
        use enumset::EnumSet;
        use kime_engine_core::{Addon, Config, Hotkey, InputCategory, Key, KeyCode::*, RawConfig};
        use shared::test_input_impl;

        #[allow(dead_code)]
        fn default_config() -> Config {
            let mut config = RawConfig::default();
            config.category_layout[InputCategory::Hangul] = $layout.into();
            Config::from_raw_config(config)
        }

        #[allow(dead_code)]
        #[track_caller]
        fn test_input(keys: &[(Key, &str, &str)]) {
            test_input_impl(&default_config(), keys);
        }

        #[allow(dead_code)]
        #[track_caller]
        fn test_word_input(keys: &[(Key, &str, &str)]) {
            let mut config = default_config();
            config.word_commit = true;
            test_input_impl(&config, keys);
        }

        #[allow(dead_code)]
        #[track_caller]
        fn test_input_with_hotkey(keys: &[(Key, &str, &str)], hotkeys: &[(Key, Hotkey)]) {
            let mut config = default_config();
            config.hotkeys = hotkeys.iter().copied().collect();
            test_input_impl(&config, keys);
        }

        #[allow(dead_code)]
        #[track_caller]
        fn test_input_with_addon(keys: &[(Key, &str, &str)], addons: impl Into<EnumSet<Addon>>) {
            let mut config = default_config();
            config.layout_addons[InputCategory::Hangul] = addons.into();
            test_input_impl(&config, keys);
        }
    };
}
