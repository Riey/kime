#![allow(clippy::missing_safety_doc)]

pub use kime_engine::{Config, InputCategory, InputEngine, InputResult, ModifierState};

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

pub const KIME_API_VERSION: usize = 4;

/// Return API version
#[no_mangle]
pub extern "C" fn kime_api_version() -> usize {
    KIME_API_VERSION
}

/// Create new engine
#[no_mangle]
pub extern "C" fn kime_engine_new(config: &Config) -> *mut InputEngine {
    Box::into_raw(Box::new(InputEngine::new(config)))
}

/// Set hangul enable state
#[no_mangle]
pub extern "C" fn kime_engine_set_input_category(
    engine: &mut InputEngine,
    category: InputCategory,
) {
    engine.set_input_category(category);
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
    engine.update_layout_state().ok();
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

/// Clear preedit state this function may append to commit string
#[no_mangle]
pub extern "C" fn kime_engine_clear_preedit(engine: &mut InputEngine) {
    engine.clear_preedit();
}

/// Clear preedit state this function must not append to commit string
#[no_mangle]
pub extern "C" fn kime_engine_remove_preedit(engine: &mut InputEngine) {
    engine.remove_preedit();
}

/// Reset engine state
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
#[cfg(unix)]
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
