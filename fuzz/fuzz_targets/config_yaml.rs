#![no_main]
use libfuzzer_sys::fuzz_target;

// Arbitrary bytes -> RawConfig deserialization must never panic: this is
// exactly what kime does with the user's config.yaml (core/src/config.rs
// load_raw_config), including the custom Key FromStr in hotkey maps.
fuzz_target!(|data: &[u8]| {
    let Ok(s) = std::str::from_utf8(data) else {
        return;
    };
    let _ = serde_yaml::from_str::<kime_engine_config::RawConfig>(s);
});
