#[macro_use]
mod shared;

define_layout_test!("dubeolsik");

#[test]
fn flexible_compose_order_addon() {
    test_input_with_addon(
        &[(Key::normal(K), "ㅏ", ""), (Key::normal(R), "가", "")],
        EnumSet::only(Addon::FlexibleComposeOrder),
    );
}

#[test]
fn strict_typing_order() {
    test_input(&[(Key::normal(K), "ㅏ", ""), (Key::normal(R), "ㄱ", "ㅏ")])
}

#[test]
fn treat_jongseong_as_choseong_compose_addon() {
    test_input_with_addon(
        &[
            (Key::normal(D), "ㅇ", ""),
            (Key::normal(M), "으", ""),
            (Key::normal(F), "을", ""),
            (Key::normal(R), "읅", ""),
            (Key::normal(R), "ㄲ", "을"),
        ],
        Addon::ComposeChoseongSsang | Addon::TreatJongseongAsChoseongCompose,
    );
}

#[test]
fn word_hello() {
    test_word_input(&[
        (Key::normal(D), "ㅇ", ""),
        (Key::normal(K), "아", ""),
        (Key::normal(S), "안", ""),
        (Key::normal(S), "안ㄴ", ""),
        (Key::normal(U), "안녀", ""),
        (Key::normal(D), "안녕", ""),
        (Key::normal(Esc), "", "안녕PASS"),
    ])
}

#[test]
fn esc() {
    test_input(&[
        (Key::normal(R), "ㄱ", ""),
        (Key::normal(Esc), "", "ㄱPASS"),
        (Key::normal(R), "", "r"),
    ]);
}

// issue #373
#[test]
fn arrow() {
    test_input(&[
        (Key::normal(R), "ㄱ", ""),
        (Key::normal(Left), "", "ㄱPASS"),
    ]);
}

#[test]
fn shift_ignore() {
    test_input(&[
        (Key::normal(R), "ㄱ", ""),
        (Key::normal(Shift), "ㄱ", ""),
        (Key::shift(O), "걔", ""),
    ])
}

#[test]
fn ctrl_w() {
    test_input(&[(Key::normal(R), "ㄱ", ""), (Key::ctrl(W), "", "ㄱPASS")]);
}

#[test]
fn next_jaum() {
    test_input(&[
        (Key::normal(D), "ㅇ", ""),
        (Key::normal(K), "아", ""),
        (Key::normal(D), "앙", ""),
        (Key::normal(E), "ㄷ", "앙"),
    ])
}

#[test]
fn next_ssangjaum() {
    test_input(&[
        (Key::normal(A), "ㅁ", ""),
        (Key::normal(K), "마", ""),
        (Key::shift(T), "맜", ""),
        (Key::normal(K), "싸", "마"),
    ])
}

#[test]
fn not_com_moum_when_continue() {
    test_input(&[
        (Key::normal(D), "ㅇ", ""),
        (Key::normal(H), "오", ""),
        (Key::normal(D), "옹", ""),
        (Key::normal(K), "아", "오"),
    ]);
}

#[test]
fn com_moum() {
    test_input(&[
        (Key::normal(D), "ㅇ", ""),
        (Key::normal(H), "오", ""),
        (Key::normal(L), "외", ""),
        (Key::normal(D), "욍", ""),
        (Key::normal(D), "ㅇ", "욍"),
        (Key::normal(K), "아", ""),
        (Key::normal(S), "안", ""),
        (Key::normal(G), "않", ""),
        (Key::normal(E), "ㄷ", "않"),
    ]);
}

#[test]
fn number() {
    test_input(&[
        (Key::normal(D), "ㅇ", ""),
        (Key::normal(H), "오", ""),
        (Key::normal(L), "외", ""),
        (Key::normal(D), "욍", ""),
        (Key::normal(D), "ㅇ", "욍"),
        (Key::normal(K), "아", ""),
        (Key::normal(S), "안", ""),
        (Key::normal(G), "않", ""),
        (Key::normal(E), "ㄷ", "않"),
        (Key::normal(One), "", "ㄷ1"),
    ]);
}

#[test]
fn exclamation_mark() {
    test_input(&[(Key::shift(R), "ㄲ", ""), (Key::shift(One), "", "ㄲ!")]);
}

#[test]
fn backspace() {
    test_input(&[
        (Key::normal(R), "ㄱ", ""),
        (Key::normal(K), "가", ""),
        (Key::normal(D), "강", ""),
        (Key::normal(Backspace), "가", ""),
        (Key::normal(Q), "갑", ""),
        (Key::normal(T), "값", ""),
        (Key::normal(Backspace), "갑", ""),
        (Key::normal(Backspace), "가", ""),
        (Key::normal(Backspace), "ㄱ", ""),
        (Key::normal(Backspace), "", ""),
        (Key::normal(D), "ㅇ", ""),
        (Key::normal(H), "오", ""),
        (Key::normal(L), "외", ""),
        (Key::normal(Backspace), "오", ""),
        (Key::normal(Backspace), "ㅇ", ""),
        (Key::normal(Backspace), "", ""),
        (Key::normal(D), "ㅇ", ""),
        (Key::normal(H), "오", ""),
        (Key::normal(K), "와", ""),
        (Key::normal(Backspace), "오", ""),
        (Key::normal(Backspace), "ㅇ", ""),
        (Key::normal(Backspace), "", ""),
        (Key::normal(R), "ㄱ", ""),
    ])
}

#[test]
fn compose_jong() {
    test_input(&[
        (Key::normal(D), "ㅇ", ""),
        (Key::normal(J), "어", ""),
        (Key::normal(Q), "업", ""),
        (Key::normal(T), "없", ""),
    ])
}

#[test]
fn backspace_moum_compose() {
    test_input(&[
        (Key::normal(D), "ㅇ", ""),
        (Key::normal(H), "오", ""),
        (Key::normal(K), "와", ""),
        (Key::normal(Backspace), "오", ""),
        (Key::normal(Backspace), "ㅇ", ""),
    ])
}
