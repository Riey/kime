#![no_main]
use libfuzzer_sys::fuzz_target;

// Arbitrary bytes -> Layout::load_from must never panic. Covers both the
// VersionProbe path and the flat/versioned schema parses, plus KeyValue
// FromStr on every map value (src/engine/backends/hangul/src/layout.rs).
fuzz_target!(|data: &[u8]| {
    let Ok(s) = std::str::from_utf8(data) else {
        return;
    };
    let _ = kime_engine_backend_hangul::Layout::load_from(s);
});
