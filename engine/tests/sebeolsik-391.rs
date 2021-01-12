const SEBEOLSIK_391_LAYOUT: &str = include_str!("../data/sebeolsik-391.yaml");

use kime_engine::{Config, InputEngine, InputResult, Key, KeyCode::*, Layout};

#[track_caller]
fn test_input(inputs: &[(Key, InputResult)]) {
    let config = Config::new(
        Layout::load_from(SEBEOLSIK_391_LAYOUT).expect("Load layout"),
        true,
        Default::default(),
        Default::default(),
        false,
    );

    let mut engine = InputEngine::new();

    engine.set_enable_hangul(true);

    for (key, expect_result) in inputs.iter().copied() {
        assert_eq!(expect_result, engine.press_key(key, &config));
    }
}

#[test]
fn hello() {
    test_input(&[
        (Key::normal(J), InputResult::Preedit('ㅇ')),
        (Key::normal(F), InputResult::Preedit('아')),
        (Key::normal(S), InputResult::Preedit('안')),
        (Key::normal(H), InputResult::CommitPreedit('안', 'ㄴ')),
        (Key::normal(E), InputResult::Preedit('녀')),
        (Key::normal(A), InputResult::Preedit('녕')),
    ])
}
