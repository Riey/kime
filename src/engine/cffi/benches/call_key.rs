#![cfg(feature = "nightly")]
#![feature(test)]

extern crate test;

use kime_engine_cffi::{Config, InputEngine};

#[bench]
fn simple(b: &mut test::Bencher) {
    let config = Config::default();
    let mut engine = InputEngine::new(&config);
    engine.set_hangul_enable(true);

    engine.press_key(&config, 52, 0);

    b.iter(|| {
        for _ in 0..1000 {
            engine.press_key(&config, 52, 0);
        }
    });
}
