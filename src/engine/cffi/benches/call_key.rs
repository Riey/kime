#![cfg(feature = "nightly")]
#![feature(test)]

extern crate test;

use kime_engine_cffi::{check_api_version, Config, InputEngine, InputCategory};

#[bench]
fn simple(b: &mut test::Bencher) {
    assert!(check_api_version());

    let config = Config::default();
    let mut engine = InputEngine::new(&config);
    engine.set_input_category(&config, InputCategory::Hangul);

    engine.press_key(&config, 52, 0);

    b.iter(|| {
        for _ in 0..1000 {
            engine.press_key(&config, 52, 0);
        }
    });
}
