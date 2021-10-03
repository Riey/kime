use kime_engine_core::{Config, EngineConfig, InputCategory, InputEngine, InputResult, Key};
use pretty_assertions::assert_eq;

#[track_caller]
pub fn test_input_impl(engine: EngineConfig, category: InputCategory, keys: &[(Key, &str, &str)]) {
    let config = Config::new(engine);
    let mut engine = InputEngine::new(&config);
    engine.set_input_category(category);

    macro_rules! test_preedit {
        ($text:expr) => {{
            assert_eq!(engine.preedit_str(), $text);
        }};
    }

    macro_rules! test_commit {
        ($text:expr) => {{
            assert_eq!(engine.commit_str(), $text);
        }};
        (@pass $text:expr) => {{
            assert_eq!(format!("{}PASS", engine.commit_str()), $text);
        }};
    }

    for (key, preedit, commit) in keys.iter().copied() {
        eprintln!("Key: {:?}", key);

        let ret = engine.press_key(key, &config);

        eprintln!("Ret: {:?}", ret);

        test_preedit!(preedit);

        if ret.contains(InputResult::HAS_COMMIT) {
            if ret.contains(InputResult::CONSUMED) {
                test_commit!(commit);
            } else {
                test_commit!(@pass commit);
            }
        } else if !ret.contains(InputResult::CONSUMED) {
            assert_eq!("PASS", commit);
        }

        engine.clear_commit();
    }
}

#[allow(unused_macros)]
macro_rules! define_layout_test {
    ($layout:expr, $latin_layout:expr, $category:expr) => {
        use kime_engine_backend_hangul::Addon;
        use kime_engine_backend_latin::LatinLayout;
        use kime_engine_core::{EngineConfig, EnumSet, Hotkey, InputCategory, Key, KeyCode::*};
        use shared::test_input_impl;

        #[allow(dead_code)]
        fn default_config() -> EngineConfig {
            let mut config = EngineConfig::default();
            config.hangul.layout = $layout.into();
            config.latin.layout = $latin_layout;
            config
        }

        #[allow(dead_code)]
        #[track_caller]
        fn test_input(keys: &[(Key, &str, &str)]) {
            test_input_impl(default_config(), $category, keys);
        }

        #[allow(dead_code)]
        #[track_caller]
        fn test_word_input(keys: &[(Key, &str, &str)]) {
            let mut config = default_config();
            config.hangul.word_commit = true;
            test_input_impl(config, $category, keys);
        }

        #[allow(dead_code)]
        #[track_caller]
        fn test_input_with_addon(keys: &[(Key, &str, &str)], addons: impl Into<EnumSet<Addon>>) {
            let mut config = default_config();
            config.hangul.addons.insert($layout.into(), addons.into());
            test_input_impl(config, $category, keys);
        }

        #[allow(dead_code)]
        #[track_caller]
        fn test_input_with_hotkey(keys: &[(Key, &str, &str)], hotkeys: &[(Key, Hotkey)]) {
            let mut config = default_config();
            config.global_hotkeys = hotkeys.iter().copied().collect();
            test_input_impl(config, $category, keys);
        }
    };
    ($layout:expr) => {
        define_layout_test!($layout, LatinLayout::Qwerty, InputCategory::Hangul);
    };
}
