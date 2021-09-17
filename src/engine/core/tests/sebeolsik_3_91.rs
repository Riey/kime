#[macro_use]
mod shared;

define_layout_test!("sebeolsik-3-91");

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

#[test]
fn switch_next() {
    test_input(&[
        (Key::normal(S), "ㄴ", ""),
        (Key::normal(J), "ㅇ", "ㄴ"),
        (Key::normal(F), "아", ""),
    ]);
}

#[test]
fn good() {
    test_input(&[
        (Key::normal(K), "ㄱ", ""),
        (Key::normal(R), "개", ""),
        (Key::normal(K), "ㄱ", "개"),
        (Key::normal(K), "ㄲ", ""),
        (Key::normal(B), "꾸", ""),
        (Key::normal(W), "꿀", ""),
    ]);
}

// https://github.com/Riey/kime/issues/521
#[test]
fn issue_521() {
    test_input(&[
        (Key::normal(J), "ㅇ", ""),
        (Key::normal(F), "아", ""),
        (Key::shift(F), "앎", ""),
    ]);
}

#[test]
fn colon() {
    test_input(&[(Key::normal(Backslash), "", ":")]);
}
