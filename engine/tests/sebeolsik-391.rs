const SEBEOLSIK_391_LAYOUT: &str = include_str!("../data/sebeolsik-391.yaml");

use kime_engine::{Config, InputEngine, InputResult, Key, KeyCode::*, Layout, RawConfig};

#[track_caller]
fn test_input(inputs: &[(Key, InputResult)]) {
    let config = Config::new(
        Layout::load_from(SEBEOLSIK_391_LAYOUT).expect("Load layout"),
        RawConfig::default(),
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
    ]);
}

#[test]
fn switch_next() {
    test_input(&[
        (Key::normal(S), InputResult::Preedit('ㄴ')),
        (Key::normal(F), InputResult::CommitPreedit('ㄴ', 'ㅏ')),
        (Key::normal(J), InputResult::Preedit('아')),
    ]);
}

#[test]
fn colon() {
    test_input(&[(Key::normal(Backslash), InputResult::Commit(':'))]);
}
