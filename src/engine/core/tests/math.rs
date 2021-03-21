#[macro_use]
mod shared;

define_layout_test!("dubeolsik", LatinLayout::Qwerty, InputCategory::Latin);

use kime_engine_core::ModifierState;

const MATH: Key = Key::new(Backslash, ModifierState::from_bits_truncate(10));

#[test]
fn twice_backspace() {
    test_input(&[
        (MATH, "", ""),
        (Key::normal(Backslash), "\\", ""),
        (Key::normal(Backslash), "", "\\"),
    ]);
}

#[test]
fn pi() {
    test_input(&[
        (MATH, "", ""),
        (Key::normal(Backslash), "\\", ""),
        (Key::normal(P), "\\p", ""),
        (Key::normal(I), "\\pi", ""),
        (Key::normal(Tab), "", "π"),
        (Key::normal(Backslash), "\\", ""),
        (Key::shift(P), "\\P", ""),
        (Key::normal(I), "\\Pi", ""),
        (Key::normal(Tab), "", "Π"),
    ]);
}

#[test]
fn space() {
    test_input(&[
        (MATH, "", ""),
        (Key::normal(Backslash), "\\", ""),
        (Key::shift(Comma), "\\<", ""),
        (Key::shift(Comma), "\\<<", ""),
        (Key::normal(Space), "", "⟪PASS"),
    ]);
}

#[test]
fn backspace() {
    test_input(&[
        (MATH, "", ""),
        (Key::normal(Backslash), "\\", ""),
        (Key::normal(P), "\\p", ""),
        (Key::normal(I), "\\pi", ""),
        (Key::normal(Backspace), "\\p", ""),
        (Key::normal(Backspace), "\\", ""),
        (Key::normal(Backspace), "", ""),
    ])
}

// issue #379
#[test]
fn esc() {
    test_input(&[
        (MATH, "", ""),
        (Key::normal(Esc), "", "PASS"),
        (Key::normal(Backslash), "", "\\"),
    ]);
}

#[test]
fn style() {
    test_input(&[
        (MATH, "", ""),
        (Key::normal(Backslash), "\\", ""),
        (Key::normal(B), "\\b", ""),
        (Key::normal(F), "\\bf", ""),
        (Key::normal(I), "\\bfi", ""),
        (Key::normal(T), "\\bfit", ""),
        (Key::normal(Period), "\\bfit.", ""),
        (Key::normal(A), "\\bfit.a", ""),
        (Key::normal(L), "\\bfit.al", ""),
        (Key::normal(P), "\\bfit.alp", ""),
        (Key::normal(H), "\\bfit.alph", ""),
        (Key::normal(A), "\\bfit.alpha", ""),
        (Key::normal(Tab), "", "𝜶"),
    ])
}
