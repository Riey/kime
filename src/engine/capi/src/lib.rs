#![allow(clippy::missing_safety_doc)]

pub use kime_engine_core::{Config, InputEngine, InputResult, ModifierState};

#[repr(C)]
pub struct XimPreeditFont {
    name: RustStr,
    size: f64,
}

#[repr(C)]
pub struct RustStr {
    ptr: *const u8,
    len: usize,
}

impl RustStr {
    pub fn new(s: &str) -> Self {
        Self {
            ptr: s.as_ptr(),
            len: s.len(),
        }
    }
}

/// Return API version
#[no_mangle]
pub extern "C" fn kime_api_version() -> usize {
    2
}

/// Create new engine
#[no_mangle]
pub extern "C" fn kime_engine_new(config: &Config) -> *mut InputEngine {
    Box::into_raw(Box::new(InputEngine::new(config.word_commit())))
}

/// Set hangul enable state
#[no_mangle]
pub extern "C" fn kime_engine_set_hangul_enable(engine: &mut InputEngine, mode: bool) {
    engine.set_hangul_enable(mode);
}

/// Delete engine
///
/// # Safety
///
/// engine must be created by `kime_engine_new` and never call delete more than once
#[no_mangle]
pub unsafe extern "C" fn kime_engine_delete(engine: &mut InputEngine) {
    drop(Box::from_raw(engine));
}

/// Update hangul state
#[no_mangle]
pub extern "C" fn kime_engine_update_hangul_state(engine: &mut InputEngine) {
    engine.update_hangul_state();
}

/// Get commit string of engine
///
/// ## Return
///
/// valid utf8 string
#[no_mangle]
pub extern "C" fn kime_engine_commit_str(engine: &mut InputEngine) -> RustStr {
    RustStr::new(engine.commit_str())
}

/// Get preedit string of engine
///
/// ## Return
///
/// valid utf8 string
#[no_mangle]
pub extern "C" fn kime_engine_preedit_str(engine: &mut InputEngine) -> RustStr {
    RustStr::new(engine.preedit_str())
}

#[no_mangle]
pub extern "C" fn kime_engine_flush(engine: &mut InputEngine) {
    engine.flush();
}

/// Reset preedit state then returm commit char
#[no_mangle]
pub extern "C" fn kime_engine_reset(engine: &mut InputEngine) {
    engine.reset();
}

/// Press key when modifier state
///
/// ## Return
///
/// input result
#[no_mangle]
pub extern "C" fn kime_engine_press_key(
    engine: &mut InputEngine,
    config: &Config,
    hardware_code: u16,
    state: ModifierState,
) -> InputResult {
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
pub extern "C" fn kime_config_xim_preedit_font(config: &Config) -> XimPreeditFont {
    let (ref font, size) = config.xim_preedit_font;

    XimPreeditFont {
        name: RustStr::new(font),
        size,
    }
}
