use kime_engine_core::{Config, InputEngine, InputResult, Key, KeyCode::*, Layout, RawConfig};

#[track_caller]
fn test_input(inputs: &[(Key, InputResult)]) {
    let config = Config::new(
        Layout::load_from(include_str!("../data/sebeolsik-sin1995.yaml")).unwrap(),
        RawConfig {
            layout: "sebeolsik-sin1995".into(),
            ..Default::default()
        },
    );

    let mut engine = InputEngine::new();

    engine.set_hangul_enable(true);

    for (key, expect_result) in inputs.iter().copied() {
        assert_eq!(engine.press_key(key, &config), expect_result);
    }
}

#[test]
fn simple() {
    test_input(&[
        (Key::normal(U), InputResult::preedit('ㄷ')),
        (Key::normal(Q), InputResult::preedit('듸')),
    ]);
}

#[test]
fn mu() {
    test_input(&[
        (Key::normal(I), InputResult::preedit('ㅁ')),
        (Key::normal(I), InputResult::preedit('무')),
    ]);
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
        (Key::normal(M), InputResult::commit_preedit('녕', 'ㅎ')),
        (Key::normal(F), InputResult::preedit('하')),
        (Key::normal(N), InputResult::commit_preedit('하', 'ㅅ')),
        (Key::normal(C), InputResult::preedit('세')),
        (Key::normal(J), InputResult::commit_preedit('세', 'ㅇ')),
        (Key::normal(X), InputResult::preedit('요')),
    ]);
}
