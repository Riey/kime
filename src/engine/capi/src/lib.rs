#![allow(clippy::missing_safety_doc)]

pub use kime_engine_core::{
    load_engine_config_from_config_dir, load_other_configs_from_config_dir, Config, DaemonConfig,
    DaemonModule, IconColor, IndicatorConfig, InputCategory, InputEngine, InputResult, LogConfig,
    ModifierState,
};

pub const KIME_API_VERSION: usize = 6;

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

#[repr(C)]
pub struct RustSlice {
    ptr: *const u8,
    len: usize,
}

impl RustSlice {
    pub fn new(s: &[u8]) -> Self {
        Self {
            ptr: s.as_ptr(),
            len: s.len(),
        }
    }
}

#[repr(C)]
pub struct FontData {
    font_data: RustSlice,
    index: u32,
    size: f32,
}

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

/// Check engine ready state
#[no_mangle]
pub unsafe extern "C" fn kime_engine_check_ready(engine: &InputEngine) -> bool {
    engine.check_ready()
}

/// End engine ready state
#[no_mangle]
pub unsafe extern "C" fn kime_engine_end_ready(engine: &mut InputEngine) -> InputResult {
    engine.end_ready()
}

/// Update layout state
#[no_mangle]
pub extern "C" fn kime_engine_update_layout_state(engine: &mut InputEngine) {
    engine.update_layout_state().ok();
}

/// Get commit string of engine
///
/// ## Return
///
/// valid utf8 string
#[no_mangle]
pub extern "C" fn kime_engine_commit_str(engine: &InputEngine) -> RustStr {
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

/// Clear commit string
#[no_mangle]
pub extern "C" fn kime_engine_clear_commit(engine: &mut InputEngine) {
    engine.clear_commit();
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
    let config = load_engine_config_from_config_dir().unwrap_or_default();
    Box::into_raw(Box::new(config))
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

/// Get candidate_font config
/// font_data only valid while config is live
#[no_mangle]
pub extern "C" fn kime_config_candidate_font(config: &Config) -> FontData {
    let (ref font, index) = config.candidate_font;

    FontData {
        font_data: RustSlice::new(font),
        index,
        size: 0.,
    }
}

/// Get xim_preedit_font config
/// font_data only valid while config is live
#[no_mangle]
pub extern "C" fn kime_config_xim_preedit_font(config: &Config) -> FontData {
    let (ref font, index, size) = config.xim_preedit_font;

    FontData {
        font_data: RustSlice::new(font),
        index,
        size,
    }
}

/// Load daemon config
#[cfg(unix)]
#[no_mangle]
pub extern "C" fn kime_daemon_config_load() -> *mut DaemonConfig {
    let config = load_other_configs_from_config_dir()
        .map(|c| c.0)
        .unwrap_or_default();
    Box::into_raw(Box::new(config))
}

/// Get daemon `modules`
#[no_mangle]
pub extern "C" fn kime_daemon_config_modules(config: &DaemonConfig) -> u32 /* enumset doesn't have transparent yet -> EnumSet<DaemonModule> */
{
    config.modules.as_u32()
}

/// Get default daemon config
#[no_mangle]
pub extern "C" fn kime_daemon_config_default() -> *mut DaemonConfig {
    Box::into_raw(Box::new(DaemonConfig::default()))
}

/// Delete daemon config
#[no_mangle]
pub unsafe extern "C" fn kime_daemon_config_delete(config: *mut DaemonConfig) {
    drop(Box::from_raw(config));
}

/// Load indicator config
#[cfg(unix)]
#[no_mangle]
pub extern "C" fn kime_indicator_config_load() -> *mut IndicatorConfig {
    let config = load_other_configs_from_config_dir()
        .map(|c| c.1)
        .unwrap_or_default();
    Box::into_raw(Box::new(config))
}

/// Get default indicator config
#[no_mangle]
pub extern "C" fn kime_indicator_config_default() -> *mut IndicatorConfig {
    Box::into_raw(Box::new(IndicatorConfig::default()))
}

/// Delete indicator config
#[no_mangle]
pub unsafe extern "C" fn kime_indicator_config_delete(config: *mut IndicatorConfig) {
    drop(Box::from_raw(config));
}

/// Get indicator `icon_color`
#[no_mangle]
pub extern "C" fn kime_indicator_config_icon_color(config: &IndicatorConfig) -> IconColor /* enumset doesn't have transparent yet -> EnumSet<DaemonModule> */
{
    config.icon_color
}

/// Load log config
#[cfg(unix)]
#[no_mangle]
pub extern "C" fn kime_log_config_load() -> *mut LogConfig {
    let config = load_other_configs_from_config_dir()
        .map(|c| c.2)
        .unwrap_or_default();
    Box::into_raw(Box::new(config))
}

/// Get default log config
#[no_mangle]
pub extern "C" fn kime_log_config_default() -> *mut LogConfig {
    Box::into_raw(Box::new(LogConfig::default()))
}

/// Delete log config
#[no_mangle]
pub unsafe extern "C" fn kime_log_config_delete(config: *mut LogConfig) {
    drop(Box::from_raw(config));
}

/// Get log `icon_color`
#[no_mangle]
pub extern "C" fn kime_log_config_global_level(config: &LogConfig) -> RustStr {
    RustStr::new(config.global_level.as_str())
}
