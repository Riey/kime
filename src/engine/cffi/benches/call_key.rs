#![cfg(feature = "nightly")]
#![feature(test)]

extern crate test;

use kime_engine_cffi::{Config, InputEngine, InputResultType};

#[bench]
fn simple(b: &mut test::Bencher) {
    let mut engine = InputEngine::new();
    let config = Config::default();
    engine.set_hangul_enable(true);

    assert_eq!(
        engine.press_key(&config, 52, 0).ty,
        InputResultType::Preedit
    );

    b.iter(|| {
        for _ in 0..1000 {
            engine.press_key(&config, 52, 0);
        }
    });
}
