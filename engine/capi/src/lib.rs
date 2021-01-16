#![allow(clippy::missing_safety_doc)]

pub use kime_engine_core::*;

#[no_mangle]
pub extern "C" fn kime_engine_new() -> *mut InputEngine {
    Box::into_raw(Box::new(InputEngine::new()))
}

#[no_mangle]
pub unsafe extern "C" fn kime_engine_delete(engine: *mut InputEngine) {
    drop(Box::from_raw(engine));
}

#[no_mangle]
pub unsafe extern "C" fn kime_engine_preedit_char(engine: *const InputEngine) -> u32 {
    let engine = engine.as_ref().unwrap();

    engine.preedit_char() as u32
}

#[no_mangle]
pub unsafe extern "C" fn kime_engine_reset(engine: *mut InputEngine) -> u32 {
    let engine = engine.as_mut().unwrap();

    engine.reset().map_or(0, |c| c as u32)
}

#[no_mangle]
pub unsafe extern "C" fn kime_engine_press_key(
    engine: *mut InputEngine,
    config: *const Config,
    hardware_code: u16,
    state: u32,
) -> InputResult {
    let engine = engine.as_mut().unwrap();
    let config = config.as_ref().unwrap();

    engine.press_key_code(hardware_code, state, config)
}

#[no_mangle]
pub extern "C" fn kime_config_load() -> *mut Config {
    Box::into_raw(Box::new(Config::load_from_config_dir().unwrap_or_default()))
}

#[no_mangle]
pub unsafe extern "C" fn kime_config_delete(config: *mut Config) {
    drop(Box::from_raw(config));
}

#[no_mangle]
pub unsafe extern "C" fn kime_config_gtk_commit_english(config: *const Config) -> u32 {
    config.as_ref().unwrap().gtk_commit_english.into()
}

#[no_mangle]
pub unsafe extern "C" fn kime_config_xim_preedit_font(
    config: *const Config,
    name: *mut *const u8,
    len: *mut usize,
) {
    let font = config.as_ref().unwrap().xim_preedit_font.as_str();
    name.write(font.as_ptr());
    len.write(font.len());
}
