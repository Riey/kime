const DUBEOLSIK_LAYOUT: &str = include_str!("../data/dubeolsik.yaml");

use kime_engine::{Config, InputEngine, InputResult, Key, KeyCode::*, Layout, RawConfig};

use pretty_assertions::assert_eq;

#[track_caller]
fn test_input(inputs: &[(Key, InputResult)]) {
    let config = Config::new(
        Layout::load_from(DUBEOLSIK_LAYOUT).expect("Load layout"),
        RawConfig::default(),
    );

    let mut engine = InputEngine::new();

    engine.set_enable_hangul(true);

    for (key, expect_result) in inputs.iter().copied() {
        assert_eq!(expect_result, engine.press_key(key, &config));
    }
}

#[test]
fn esc() {
    test_input(&[
        (Key::normal(R), InputResult::Preedit('ㄱ')),
        (Key::normal(Esc), InputResult::CommitBypass('ㄱ')),
        (Key::normal(R), InputResult::Bypass),
    ]);
}

#[test]
fn issue_28() {
    test_input(&[
        (Key::normal(K), InputResult::Preedit('ㅏ')),
        (Key::normal(R), InputResult::Preedit('가')),
    ])
}

#[test]
fn next_jaum() {
    test_input(&[
        (Key::normal(D), InputResult::Preedit('ㅇ')),
        (Key::normal(K), InputResult::Preedit('아')),
        (Key::normal(D), InputResult::Preedit('앙')),
        (Key::normal(E), InputResult::CommitPreedit('앙', 'ㄷ')),
    ])
}

#[test]
fn not_com_moum_when_continue() {
    test_input(&[
        (Key::normal(D), InputResult::Preedit('ㅇ')),
        (Key::normal(H), InputResult::Preedit('오')),
        (Key::normal(D), InputResult::Preedit('옹')),
        (Key::normal(K), InputResult::CommitPreedit('오', '아')),
    ]);
}

#[test]
fn com_moum() {
    test_input(&[
        (Key::normal(D), InputResult::Preedit('ㅇ')),
        (Key::normal(H), InputResult::Preedit('오')),
        (Key::normal(L), InputResult::Preedit('외')),
        (Key::normal(D), InputResult::Preedit('욍')),
        (Key::normal(D), InputResult::CommitPreedit('욍', 'ㅇ')),
        (Key::normal(K), InputResult::Preedit('아')),
        (Key::normal(S), InputResult::Preedit('안')),
        (Key::normal(G), InputResult::Preedit('않')),
        (Key::normal(E), InputResult::CommitPreedit('않', 'ㄷ')),
    ]);
}

#[test]
fn number() {
    test_input(&[
        (Key::normal(D), InputResult::Preedit('ㅇ')),
        (Key::normal(H), InputResult::Preedit('오')),
        (Key::normal(L), InputResult::Preedit('외')),
        (Key::normal(D), InputResult::Preedit('욍')),
        (Key::normal(D), InputResult::CommitPreedit('욍', 'ㅇ')),
        (Key::normal(K), InputResult::Preedit('아')),
        (Key::normal(S), InputResult::Preedit('안')),
        (Key::normal(G), InputResult::Preedit('않')),
        (Key::normal(E), InputResult::CommitPreedit('않', 'ㄷ')),
        (Key::normal(One), InputResult::CommitCommit('ㄷ', '1')),
    ]);
}

#[test]
fn exclamation_mark() {
    test_input(&[
        (Key::shift(R), InputResult::Preedit('ㄲ')),
        (Key::shift(One), InputResult::CommitCommit('ㄲ', '!')),
    ]);
}

#[test]
fn backspace() {
    test_input(&[
        (Key::normal(R), InputResult::Preedit('ㄱ')),
        (Key::normal(K), InputResult::Preedit('가')),
        (Key::normal(D), InputResult::Preedit('강')),
        (Key::normal(Backspace), InputResult::Preedit('가')),
        (Key::normal(Q), InputResult::Preedit('갑')),
        (Key::normal(T), InputResult::Preedit('값')),
        (Key::normal(Backspace), InputResult::Preedit('갑')),
        (Key::normal(Backspace), InputResult::Preedit('가')),
        (Key::normal(Backspace), InputResult::Preedit('ㄱ')),
        (Key::normal(Backspace), InputResult::ClearPreedit),
        (Key::normal(R), InputResult::Preedit('ㄱ')),
    ])
}

#[test]
fn compose_jong() {
    test_input(&[
        (Key::normal(D), InputResult::Preedit('ㅇ')),
        (Key::normal(J), InputResult::Preedit('어')),
        (Key::normal(Q), InputResult::Preedit('업')),
        (Key::normal(T), InputResult::Preedit('없')),
    ])
}

#[test]
fn backspace_moum_compose() {
    test_input(&[
        (Key::normal(D), InputResult::Preedit('ㅇ')),
        (Key::normal(H), InputResult::Preedit('오')),
        (Key::normal(K), InputResult::Preedit('와')),
        (Key::normal(Backspace), InputResult::Preedit('오')),
        (Key::normal(Backspace), InputResult::Preedit('ㅇ')),
    ])
}
