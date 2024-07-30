#[macro_use]
mod shared;

define_layout_test!("dubeolsik", LatinLayout::Qwerty, InputCategory::Latin);

use kime_engine_core::ModifierState;

const EMOJI: Key = Key::new(E, ModifierState::CONTROL.union(ModifierState::ALT));

#[test]
fn thinking() {
    test_input(&[
        (EMOJI, "🏻(light skin tone)🏼(medium-light skin tone)🏽(medium skin tone)🏾(medium-dark skin tone)🏿(dark skin tone)", ""),
        (Key::normal(T), "t🏻(light skin tone)🏼(medium-light skin tone)🏽(medium skin tone)🏾(medium-dark skin tone)🏿(dark skin tone)", ""),
        (Key::normal(H), "th😃(grinning face with big eyes)😄(grinning face with smiling eyes)😁(beaming face with smiling eyes)😅(grinning face with sweat)🤣(rolling on the floor laughing)", ""),
        (Key::normal(I), "thi🤔(thinking face)🕧(twelve-thirty)🕜(one-thirty)🕝(two-thirty)🕞(three-thirty)", ""),
        (Key::normal(N), "thin🤔(thinking face)", ""),
        (Key::normal(K), "think🤔(thinking face)", ""),
        (Key::normal(Enter), "", "🤔"),
    ]);
}
