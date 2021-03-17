#[macro_use]
mod shared;

define_layout_test!("dubeolsik", LatinLayout::Qwerty, InputCategory::Latin);

use kime_engine_core::ModifierState;

const EMOJI: Key = Key::new(E, ModifierState::from_bits_truncate(9));

#[test]
fn thinking() {
    test_input(&[
        (EMOJI, "🏻(light skin tone)🏼(medium-light skin tone)🏽(medium skin tone)🏾(medium-dark skin tone)🏿(dark skin tone)", ""),
        (Key::normal(T), "t🏻(light skin tone)🏼(medium-light skin tone)🏽(medium skin tone)🏾(medium-dark skin tone)🏿(dark skin tone)", ""),
        (Key::normal(H), "th😁(beaming face with smiling eyes)😂(face with tears of joy)🤣(rolling on the floor laughing)😃(grinning face with big eyes)😄(grinning face with smiling eyes)", ""),
        (Key::normal(I), "thi🤔(thinking face)🕧(twelve-thirty)🕜(one-thirty)🕝(two-thirty)🕞(three-thirty)", ""),
        (Key::normal(N), "thin🤔(thinking face)", ""),
        (Key::normal(K), "think🤔(thinking face)", ""),
        (Key::normal(Enter), "", "🤔"),
    ]);
}
