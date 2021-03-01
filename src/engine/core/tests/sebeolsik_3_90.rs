#[macro_use]
mod shared;

define_layout_test!("sebeolsik-3-90");

#[test]
fn hello() {
    test_input(&[
        (Key::normal(J), "ㅇ", ""),
        (Key::normal(F), "아", ""),
        (Key::normal(S), "안", ""),
        (Key::normal(H), "ㄴ", "안"),
        (Key::normal(E), "녀", ""),
        (Key::normal(A), "녕", ""),
    ]);
}

// issue #263
#[test]
fn compose_choseong_ssang() {
    test_input(&[
        (Key::normal(K), "ㄱ", ""),
        (Key::normal(F), "가", ""),
        (Key::normal(X), "각", ""),
        (Key::normal(K), "ㄱ", "각"),
        (Key::normal(D), "기", ""),
    ])
}

#[test]
fn switch_next() {
    test_input(&[
        (Key::normal(S), "ㄴ", ""),
        (Key::normal(J), "ㅇ", "ㄴ"),
        (Key::normal(F), "아", ""),
    ]);
}

#[test]
fn dont_change_jongseong() {
    test_input(&[
        (Key::normal(J), "ㅇ", ""),
        (Key::normal(F), "아", ""),
        (Key::normal(S), "안", ""),
        (Key::normal(D), "ㅣ", "안"),
    ]);
}

#[test]
fn s_number() {
    test_input(&[
        (Key::shift(Two), "", "@"),
        (Key::shift(Three), "", "#"),
        (Key::shift(Four), "", "$"),
        (Key::shift(Five), "", "%"),
        (Key::shift(Six), "", "^"),
        (Key::shift(Seven), "", "&"),
        (Key::shift(Eight), "", "*"),
        (Key::shift(Nine), "", "("),
        (Key::shift(Zero), "", ")"),
    ])
}

#[test]
fn colon() {
    test_input(&[(Key::shift(SemiColon), "", ":")]);
}
