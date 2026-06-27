use kime_engine_config::{HotkeyBehavior, HotkeyResult};
use kime_engine_core::{
    Config, EngineConfig, Hotkey, InputCategory, InputEngine, InputResult, Key, KeyCode,
    ModifierState,
};
use pretty_assertions::assert_eq;

// X11 hardware keycodes (evdev + 8) for the Super keys.
const SUPER_R: u16 = 134;
// R key, which is `ㄱ` in dubeolsik (the default hangul layout).
const KEY_R: u16 = 27;

/// Right-Super (Apple right-command) can be bound as a category toggle, the
/// exact use case requested in issue #640. This drives the full hardware-code
/// path so it also proves `from_hardware_code(134)` resolves to `SuperR`.
#[test]
fn super_r_can_toggle_category() {
    let mut engine_config = EngineConfig::default();
    engine_config.global_hotkeys = std::iter::once((
        Key::normal(KeyCode::SuperR),
        Hotkey::new(HotkeyBehavior::toggle_hangul_latin(), HotkeyResult::Consume),
    ))
    .collect();

    let config = Config::new(engine_config);
    let mut engine = InputEngine::new(&config);
    engine.set_input_category(InputCategory::Hangul);
    assert_eq!(engine.category(), InputCategory::Hangul);

    let ret = engine.press_key_code(SUPER_R, ModifierState::empty(), false, &config);
    assert!(ret.contains(InputResult::CONSUMED));
    assert_eq!(engine.category(), InputCategory::Latin);

    engine.press_key_code(SUPER_R, ModifierState::empty(), false, &config);
    assert_eq!(engine.category(), InputCategory::Hangul);
}

/// An unbound Super key press must behave like the other modifier keys
/// (Alt/Control) and leave an in-progress preedit untouched instead of
/// clearing it.
#[test]
fn unbound_super_keeps_preedit() {
    let config = Config::new(EngineConfig::default());
    let mut engine = InputEngine::new(&config);
    engine.set_input_category(InputCategory::Hangul);

    engine.press_key_code(KEY_R, ModifierState::empty(), false, &config);
    assert_eq!(engine.preedit_str(), "ㄱ");

    engine.press_key_code(SUPER_R, ModifierState::empty(), false, &config);
    assert_eq!(engine.preedit_str(), "ㄱ");
}
