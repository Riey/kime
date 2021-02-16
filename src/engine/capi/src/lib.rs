#![allow(clippy::missing_safety_doc)]

pub use kime_engine_core::{Config, InputEngine, InputResult, ModifierState};

// panic-safe version of ptr to ref

macro_rules! to_mut {
    ($val:ident) => {
        let $val = match $val.as_mut() {
            Some(r) => r,
            None => return Default::default(),
        };
    };
}

macro_rules! to_ref {
    ($val:ident) => {
        let $val = match $val.as_ref() {
            Some(r) => r,
            None => return Default::default(),
        };
    };
}

/// Return API version
#[no_mangle]
pub extern "C" fn kime_api_version() -> usize {
    1
}

/// Create new engine
#[no_mangle]
pub extern "C" fn kime_engine_new() -> *mut InputEngine {
    Box::into_raw(Box::new(InputEngine::new()))
}

/// Set hangul enable state
#[no_mangle]
pub unsafe extern "C" fn kime_engine_set_hangul_enable(engine: *mut InputEngine, mode: bool) {
    to_mut!(engine);
    engine.set_hangul_enable(mode);
}

/// Delete engine
///
/// # Safety
///
/// engine must be created by `kime_engine_new` and never call delete more than once
#[no_mangle]
pub unsafe extern "C" fn kime_engine_delete(engine: *mut InputEngine) {
    drop(Box::from_raw(engine));
}

/// Update hangul state
#[no_mangle]
pub unsafe extern "C" fn kime_engine_update_hangul_state(engine: *mut InputEngine) {
    to_mut!(engine);
    engine.update_hangul_state();
}

/// Get preedit_char of engine
///
/// ## Return
///
/// valid ucs4 char NULL to represent empty
#[no_mangle]
pub unsafe extern "C" fn kime_engine_preedit_char(engine: *const InputEngine) -> u32 {
    to_ref!(engine);
    engine.preedit_char() as u32
}

/// Reset preedit state then returm commit char
///
/// ## Return
///
/// valid ucs4 char NULL to represent empty
#[no_mangle]
pub unsafe extern "C" fn kime_engine_reset(engine: *mut InputEngine) -> u32 {
    to_mut!(engine);
    engine.reset() as u32
}

/// Press key when modifier state
///
/// ## Return
///
/// input result
#[no_mangle]
pub unsafe extern "C" fn kime_engine_press_key(
    engine: *mut InputEngine,
    config: *const Config,
    hardware_code: u16,
    state: ModifierState,
) -> InputResult {
    to_mut!(engine);
    to_ref!(config);

    engine.press_key_code(hardware_code, state, config)
}

/// Load config from local file
#[no_mangle]
pub extern "C" fn kime_config_load() -> *mut Config {
    Box::into_raw(Box::new(Config::load_from_config_dir().unwrap_or_default()))
}

/// Create default config note that this function will not read config file
#[no_mangle]
pub extern "C" fn kime_config_default() -> *mut Config {
    Box::into_raw(Box::new(Config::default()))
}

/// Delete config
#[no_mangle]
pub unsafe extern "C" fn kime_config_delete(config: *mut Config) {
    drop(Box::from_raw(config));
}

/// Get xim_preedit_font config
/// name only valid while config is live
///
/// ## Return
///
/// utf-8 string when len
#[no_mangle]
pub unsafe extern "C" fn kime_config_xim_preedit_font(
    config: *const Config,
    name: *mut *const u8,
    len: *mut usize,
    font_size: *mut f64,
) {
    to_ref!(config);
    let (ref font, size) = config.xim_preedit_font;
    name.write(font.as_ptr());
    len.write(font.len());
    font_size.write(size);
}
