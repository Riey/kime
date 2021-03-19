#![cfg(feature = "nightly")]
#![feature(test)]

extern crate test;

use kime_engine_core::{Config, InputCategory, InputEngine, Key, KeyCode::*};

#[bench]
fn simple(b: &mut test::Bencher) {
    let config = Config::default();
    let mut engine = InputEngine::new(&config);
    engine.set_input_category(InputCategory::Hangul);

    b.bytes += 1000;

    // warm up
    for _ in 0..100 {
        engine.press_key(Key::normal(A), &config);
        engine.press_key(Key::normal(Backspace), &config);
    }

    b.iter(|| {
        for _ in 0..1000 {
            engine.press_key(Key::normal(A), &config);
            engine.press_key(Key::normal(Backspace), &config);
        }
    });
}
