#[macro_use]
mod shared;

define_layout_test!("dubeolsik", LatinLayout::Qwerty, InputCategory::Latin);

#[test]
fn qwerty() {
    test_input(&[
        (Key::normal(A), "", "PASS"),
        (Key::normal(S), "", "PASS"),
        (Key::shift(SemiColon), "", "PASS"),
    ]);
}
