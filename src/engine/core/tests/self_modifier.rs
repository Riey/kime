use kime_engine_config::{HotkeyBehavior, HotkeyResult};
use kime_engine_core::{
    Config, EngineConfig, Hotkey, InputCategory, InputEngine, InputResult, Key, KeyCode,
    ModifierState,
};
use pretty_assertions::assert_eq;

// X11 hardware keycodes (evdev + 8).
const ALT_R: u16 = 108;
const CONTROL_R: u16 = 105;
const KEY_E: u16 = 26;

fn toggle_config(hotkey: Key) -> Config {
    let mut engine_config = EngineConfig::default();
    engine_config.global_hotkeys = std::iter::once((
        hotkey,
        Hotkey::new(HotkeyBehavior::toggle_hangul_latin(), HotkeyResult::Consume),
    ))
    .collect();
    // The default category hotkeys bind ControlR to Hanja mode, which would
    // shadow the global toggle under test.
    engine_config.category_hotkeys.clear();
    Config::new(engine_config)
}

/// On Wayland the press of a modifier key is delivered with its own modifier
/// bit already set (X11 reports the pre-event state), e.g. AltR arrives as
/// `Key { AltR, ALT }`. A config entry written as plain `AltR` must still
/// match.
#[test]
fn altr_toggle_matches_when_own_modifier_is_set() {
    let config = toggle_config(Key::normal(KeyCode::AltR));
    let mut engine = InputEngine::new(&config);
    engine.set_input_category(InputCategory::Hangul);

    let ret = engine.press_key_code(ALT_R, ModifierState::ALT, false, &config);
    assert!(ret.contains(InputResult::CONSUMED));
    assert_eq!(engine.category(), InputCategory::Latin);
}

/// Same as the AltR case for the right control key, which is commonly bound
/// as the Hanja hotkey.
#[test]
fn control_r_hotkey_matches_when_own_modifier_is_set() {
    let config = toggle_config(Key::normal(KeyCode::ControlR));
    let mut engine = InputEngine::new(&config);
    engine.set_input_category(InputCategory::Hangul);

    let ret = engine.press_key_code(CONTROL_R, ModifierState::CONTROL, false, &config);
    assert!(ret.contains(InputResult::CONSUMED));
    assert_eq!(engine.category(), InputCategory::Latin);
}

/// Only the key's own modifier bit may be dropped: `C-AltR` pressed while
/// control is held arrives as CONTROL|ALT and must still match.
#[test]
fn combo_hotkey_on_modifier_key_still_matches() {
    let config = toggle_config(Key::ctrl(KeyCode::AltR));
    let mut engine = InputEngine::new(&config);
    engine.set_input_category(InputCategory::Hangul);

    let ret = engine.press_key_code(
        ALT_R,
        ModifierState::CONTROL | ModifierState::ALT,
        false,
        &config,
    );
    assert!(ret.contains(InputResult::CONSUMED));
    assert_eq!(engine.category(), InputCategory::Latin);
}

/// A config that binds only `M-AltR` (the pre-existing Wayland workaround)
/// must keep working: the exact modified key is looked up before the
/// self-modifier fallback.
#[test]
fn explicit_same_class_binding_stays_reachable() {
    let config = toggle_config(Key::alt(KeyCode::AltR));
    let mut engine = InputEngine::new(&config);
    engine.set_input_category(InputCategory::Hangul);

    let ret = engine.press_key_code(ALT_R, ModifierState::ALT, false, &config);
    assert!(ret.contains(InputResult::CONSUMED));
    assert_eq!(engine.category(), InputCategory::Latin);
}

/// When both `AltR` and `M-AltR` are bound, the exact match must win over
/// the self-modifier fallback.
#[test]
fn exact_match_beats_self_modifier_fallback() {
    let mut engine_config = EngineConfig::default();
    engine_config.global_hotkeys = [
        (
            Key::normal(KeyCode::AltR),
            Hotkey::new(
                HotkeyBehavior::Switch(InputCategory::Hangul),
                HotkeyResult::Consume,
            ),
        ),
        (
            Key::alt(KeyCode::AltR),
            Hotkey::new(
                HotkeyBehavior::Switch(InputCategory::Latin),
                HotkeyResult::Consume,
            ),
        ),
    ]
    .into_iter()
    .collect();
    engine_config.category_hotkeys.clear();
    let config = Config::new(engine_config);
    let mut engine = InputEngine::new(&config);
    engine.set_input_category(InputCategory::Hangul);

    engine.press_key_code(ALT_R, ModifierState::ALT, false, &config);
    assert_eq!(engine.category(), InputCategory::Latin);
}

/// Non-modifier keys must keep their full state: `M-E` style hotkeys rely on
/// the ALT bit staying put when E is pressed.
#[test]
fn non_modifier_key_state_is_untouched() {
    let config = toggle_config(Key::alt(KeyCode::E));
    let mut engine = InputEngine::new(&config);
    engine.set_input_category(InputCategory::Hangul);

    let ret = engine.press_key_code(KEY_E, ModifierState::ALT, false, &config);
    assert!(ret.contains(InputResult::CONSUMED));
    assert_eq!(engine.category(), InputCategory::Latin);
}
