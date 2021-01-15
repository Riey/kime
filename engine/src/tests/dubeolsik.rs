const DUBEOLSIK_LAYOUT: &str = include_str!("../../data/dubeolsik.yaml");

use crate::{Config, InputEngine, InputResult, Key, KeyCode::*, Layout};

#[track_caller]
fn test_input(inputs: &[(Key, InputResult)]) {
    let config = Config::new(
        Layout::load_from(DUBEOLSIK_LAYOUT).expect("Load layout"),
        Default::default(),
    );

    let mut engine = InputEngine::new();

    engine.set_enable_hangul(true);

    for (key, expect_result) in inputs.iter().copied() {
        assert_eq!(
            expect_result,
            engine.press_key(key, &config),
            "key: {}",
            key
        );
    }
}

#[test]
fn esc() {
    test_input(&[
        (Key::normal(R), InputResult::preedit('ㄱ')),
        (Key::normal(Esc), InputResult::commit_bypass('ㄱ')),
        (Key::normal(R), InputResult::bypass()),
    ]);
}

#[test]
fn issue_28() {
    test_input(&[
        (Key::normal(K), InputResult::preedit('ㅏ')),
        (Key::normal(R), InputResult::preedit('가')),
    ])
}

#[test]
fn next_jaum() {
    test_input(&[
        (Key::normal(D), InputResult::preedit('ㅇ')),
        (Key::normal(K), InputResult::preedit('아')),
        (Key::normal(D), InputResult::preedit('앙')),
        (Key::normal(E), InputResult::commit_preedit('앙', 'ㄷ')),
    ])
}

#[test]
fn next_ssangjaum() {
    test_input(&[
        (Key::normal(A), InputResult::preedit('ㅁ')),
        (Key::normal(K), InputResult::preedit('마')),
        (Key::shift(T), InputResult::preedit('맜')),
        (Key::normal(K), InputResult::commit_preedit('마', '싸')),
    ])
}

#[test]
fn not_com_moum_when_continue() {
    test_input(&[
        (Key::normal(D), InputResult::preedit('ㅇ')),
        (Key::normal(H), InputResult::preedit('오')),
        (Key::normal(D), InputResult::preedit('옹')),
        (Key::normal(K), InputResult::commit_preedit('오', '아')),
    ]);
}

#[test]
fn com_moum() {
    test_input(&[
        (Key::normal(D), InputResult::preedit('ㅇ')),
        (Key::normal(H), InputResult::preedit('오')),
        (Key::normal(L), InputResult::preedit('외')),
        (Key::normal(D), InputResult::preedit('욍')),
        (Key::normal(D), InputResult::commit_preedit('욍', 'ㅇ')),
        (Key::normal(K), InputResult::preedit('아')),
        (Key::normal(S), InputResult::preedit('안')),
        (Key::normal(G), InputResult::preedit('않')),
        (Key::normal(E), InputResult::commit_preedit('않', 'ㄷ')),
    ]);
}

#[test]
fn number() {
    test_input(&[
        (Key::normal(D), InputResult::preedit('ㅇ')),
        (Key::normal(H), InputResult::preedit('오')),
        (Key::normal(L), InputResult::preedit('외')),
        (Key::normal(D), InputResult::preedit('욍')),
        (Key::normal(D), InputResult::commit_preedit('욍', 'ㅇ')),
        (Key::normal(K), InputResult::preedit('아')),
        (Key::normal(S), InputResult::preedit('안')),
        (Key::normal(G), InputResult::preedit('않')),
        (Key::normal(E), InputResult::commit_preedit('않', 'ㄷ')),
        (Key::normal(One), InputResult::commit2('ㄷ', '1')),
    ]);
}

#[test]
fn exclamation_mark() {
    test_input(&[
        (Key::shift(R), InputResult::preedit('ㄲ')),
        (Key::shift(One), InputResult::commit2('ㄲ', '!')),
    ]);
}

#[test]
fn backspace() {
    test_input(&[
        (Key::normal(R), InputResult::preedit('ㄱ')),
        (Key::normal(K), InputResult::preedit('가')),
        (Key::normal(D), InputResult::preedit('강')),
        (Key::normal(Backspace), InputResult::preedit('가')),
        (Key::normal(Q), InputResult::preedit('갑')),
        (Key::normal(T), InputResult::preedit('값')),
        (Key::normal(Backspace), InputResult::preedit('갑')),
        (Key::normal(Backspace), InputResult::preedit('가')),
        (Key::normal(Backspace), InputResult::preedit('ㄱ')),
        (Key::normal(Backspace), InputResult::clear_preedit()),
        (Key::normal(D), InputResult::preedit('ㅇ')),
        (Key::normal(H), InputResult::preedit('오')),
        (Key::normal(K), InputResult::preedit('와')),
        (Key::normal(Backspace), InputResult::preedit('오')),
        (Key::normal(Backspace), InputResult::preedit('ㅇ')),
        (Key::normal(Backspace), InputResult::clear_preedit()),
        (Key::normal(R), InputResult::preedit('ㄱ')),
    ])
}

#[test]
fn compose_jong() {
    test_input(&[
        (Key::normal(D), InputResult::preedit('ㅇ')),
        (Key::normal(J), InputResult::preedit('어')),
        (Key::normal(Q), InputResult::preedit('업')),
        (Key::normal(T), InputResult::preedit('없')),
    ])
}

#[test]
fn backspace_moum_compose() {
    test_input(&[
        (Key::normal(D), InputResult::preedit('ㅇ')),
        (Key::normal(H), InputResult::preedit('오')),
        (Key::normal(K), InputResult::preedit('와')),
        (Key::normal(Backspace), InputResult::preedit('오')),
        (Key::normal(Backspace), InputResult::preedit('ㅇ')),
    ])
}
