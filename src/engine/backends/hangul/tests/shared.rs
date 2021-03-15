use kime_engine_core::{InputEngine, InputResult, Key};
use kime_engine_hangul::{builtin_layouts, HangulConfig, HangulEngine};

#[track_caller]
pub fn test_input_impl(config: &HangulConfig, keys: &[(Key, &str, &str)]) {
    let mut engine = HangulEngine::new(config, builtin_layouts());

    let mut buf = String::with_capacity(16);

    macro_rules! test_preedit {
        ($text:expr) => {{
            buf.clear();
            engine.preedit_str(&mut buf);
            assert_eq!(buf.as_str(), $text);
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

        let ret = engine.press_key(key);

        eprintln!("Ret: {:?}", ret);

        if engine.has_preedit() {
            test_preedit!(preedit);
        } else {
            assert!(preedit.is_empty());
        }

        if !ret {
            test_commit!(@pass commit);
        } else {
            test_commit!(commit);
            engine.clear_commit();
        }
    }
}

#[allow(unused_macros)]
macro_rules! define_layout_test {
    ($layout:literal) => {
        use enumset::EnumSet;
        use kime_engine_core::{Key, KeyCode::*};
        use kime_engine_hangul::{Addon, HangulConfig};
        use shared::test_input_impl;

        #[allow(dead_code)]
        fn default_config() -> HangulConfig {
            let mut config = HangulConfig::default();
            config.layout = $layout.into();
            config
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
        fn test_input_with_addon(keys: &[(Key, &str, &str)], addons: impl Into<EnumSet<Addon>>) {
            let mut config = default_config();
            config.addons.insert($layout.into(), addons.into());
            test_input_impl(&config, keys);
        }
    };
}
