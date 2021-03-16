#[macro_use]
mod shared;

define_layout_test!("dubeolsik", LatinLayout::Qwerty, InputCategory::Math);

#[test]
fn twice_backspace() {
    test_input(&[
        (Key::normal(Backslash), "\\", ""),
        (Key::normal(Backslash), "", "\\"),
    ]);
}

#[test]
fn pi() {
    test_input(&[
        (Key::normal(Backslash), "\\", ""),
        (Key::normal(P), "\\p", ""),
        (Key::normal(I), "\\pi", ""),
        (Key::normal(Enter), "", "π"),
        (Key::normal(Backslash), "\\", ""),
        (Key::shift(P), "\\P", ""),
        (Key::normal(I), "\\Pi", ""),
        (Key::normal(Enter), "", "Π"),
    ]);
}

#[test]
fn space() {
    test_input(&[
        (Key::normal(Backslash), "\\", ""),
        (Key::shift(Comma), "\\<", ""),
        (Key::shift(Comma), "\\<<", ""),
        (Key::normal(Space), "", "⟪PASS"),
    ]);
}
