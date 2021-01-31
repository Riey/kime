use kime_engine_core::{Config, InputEngine, InputResult, Key, KeyCode::*, RawConfig};

#[track_caller]
fn test_input(inputs: &[(Key, InputResult)]) {
    let config = Config::from_raw_config(
        RawConfig {
            layout: "sebeolsik-391".into(),
            ..Default::default()
        },
        None,
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
        (Key::normal(J), InputResult::preedit('ㅇ')),
        (Key::normal(F), InputResult::preedit('아')),
        (Key::normal(S), InputResult::preedit('안')),
        (Key::normal(H), InputResult::commit_preedit('안', 'ㄴ')),
        (Key::normal(E), InputResult::preedit('녀')),
        (Key::normal(A), InputResult::preedit('녕')),
    ]);
}

#[test]
fn switch_next() {
    test_input(&[
        (Key::normal(S), InputResult::preedit('ㄴ')),
        (Key::normal(F), InputResult::commit_preedit('ㄴ', 'ㅏ')),
        (Key::normal(J), InputResult::preedit('아')),
    ]);
}

#[test]
fn colon() {
    test_input(&[(Key::normal(Backslash), InputResult::commit(':'))]);
}
