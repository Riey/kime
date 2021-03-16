#[macro_use]
mod shared;

define_layout_test!("sebeolsik-3sin-p2");

#[test]
fn simple() {
    test_input(&[(Key::normal(U), "ㄷ", ""), (Key::normal(Z), "듸", "")]);
}

#[test]
fn cu() {
    test_input(&[(Key::normal(O), "ㅊ", ""), (Key::normal(O), "추", "")]);
}

#[test]
fn hello() {
    test_input(&[
        (Key::normal(J), "ㅇ", ""),
        (Key::normal(F), "아", ""),
        (Key::normal(S), "안", ""),
        (Key::normal(H), "ㄴ", "안"),
        (Key::normal(T), "녀", ""),
        (Key::normal(A), "녕", ""),
        (Key::normal(M), "ㅎ", "녕"),
        (Key::normal(F), "하", ""),
        (Key::normal(N), "ㅅ", "하"),
        (Key::normal(C), "세", ""),
        (Key::normal(J), "ㅇ", "세"),
        (Key::normal(X), "요", ""),
    ]);
}

#[test]
fn compose_jungseong() {
    test_input(&[
        (Key::normal(J), "ㅇ", ""),
        (Key::normal(O), "우", ""),
        (Key::normal(C), "웨", ""),
        (Key::normal(S), "웬", ""),
        (Key::normal(J), "ㅇ", "웬"),
        (Key::normal(Slash), "오", ""),
        (Key::normal(D), "외", ""),
    ]);
}

#[test]
fn dont_compose_jungseong() {
    test_input(&[
        (Key::normal(J), "ㅇ", ""),
        (Key::normal(B), "우", ""),
        (Key::normal(B), "웇", ""),
        (Key::normal(J), "ㅇ", "웇"),
        (Key::normal(V), "오", ""),
        (Key::normal(D), "옿", ""),
    ]);
}

#[test]
fn chocolate() {
    test_input(&[
        (Key::normal(J), "ㅇ", ""),
        (Key::normal(O), "우", ""),
        (Key::normal(C), "웨", ""),
        (Key::normal(S), "웬", ""),
        (Key::normal(O), "ㅊ", "웬"),
        (Key::normal(V), "초", ""),
        (Key::normal(Slash), "ㅋ", "초"),
        (Key::normal(V), "코", ""),
        (Key::normal(W), "콜", ""),
        (Key::normal(Y), "ㄹ", "콜"),
        (Key::normal(D), "리", ""),
        (Key::normal(Q), "릿", ""),
    ]);
}
