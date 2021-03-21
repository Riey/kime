use criterion::{criterion_group, criterion_main, Criterion};
use kime_engine_core::{Config, InputCategory, InputEngine, Key, KeyCode::*};

fn simple(c: &mut Criterion) {
    let config = Config::default();

    c.bench_function("simple 100", |b| {
        let mut engine = InputEngine::new(&config);
        engine.set_input_category(InputCategory::Hangul);
        b.iter(|| {
            for _ in 0..100 {
                engine.press_key(Key::normal(A), &config);
                engine.press_key(Key::normal(Backspace), &config);
            }
        })
    });

    c.bench_function("simple 1000", |b| {
        let mut engine = InputEngine::new(&config);
        engine.set_input_category(InputCategory::Hangul);
        b.iter(|| {
            for _ in 0..1000 {
                engine.press_key(Key::normal(A), &config);
                engine.press_key(Key::normal(Backspace), &config);
            }
        })
    });
}

criterion_group!(benches, simple);
criterion_main!(benches);
