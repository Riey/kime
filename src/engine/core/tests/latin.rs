#[macro_use]
mod shared;

define_layout_test!("dubeolsik", LatinLayout::Qwerty, InputCategory::Latin);

#[test]
fn qwert() {
    test_input(&[
        (Key::normal(A), "", "a"),
        (Key::normal(S), "", "s"),
        (Key::shift(SemiColon), "", ":"),
    ]);
}
