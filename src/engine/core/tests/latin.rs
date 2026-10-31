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

fn dvorak_config(preferred_direct: bool) -> EngineConfig {
    let mut config = EngineConfig::default();
    config.latin.layout = LatinLayout::Dvorak;
    config.latin.preferred_direct = preferred_direct;
    config
}

/// With `preferred_direct: false` the embedded Dvorak layout is applied:
/// physical QWERTY positions are remapped to their Dvorak characters.
/// (`W` -> `,` used to be impossible because dvorak.yaml failed to parse, #626.)
#[test]
fn dvorak_applies_when_not_preferred_direct() {
    test_input_impl(
        dvorak_config(false),
        InputCategory::Latin,
        &[
            (Key::normal(Q), "", "'"),
            (Key::normal(W), "", ","),
            (Key::shift(W), "", "<"),
            (Key::normal(E), "", "."),
            (Key::normal(K), "", "t"),
            (Key::shift(Z), "", ";"),
        ],
    );
}

/// Reproduces the exact #626 configuration (Dvorak + `preferred_direct: true`,
/// which is the default): keys are passed through to the OS layout untouched,
/// so the embedded Dvorak layout has no effect.
#[test]
fn dvorak_ignored_when_preferred_direct() {
    test_input_impl(
        dvorak_config(true),
        InputCategory::Latin,
        &[
            (Key::normal(Q), "", "PASS"),
            (Key::normal(W), "", "PASS"),
            (Key::normal(K), "", "PASS"),
        ],
    );
}
