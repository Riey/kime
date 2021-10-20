#[allow(dead_code)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
mod ffi {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

#[link(name = "kime_engine", kind = "dylib")]
extern "C" {}

pub use kime_engine_config::{DaemonModule, EnumSet};

pub use ffi::{
    IconColor, InputCategory, InputResult, InputResult_CONSUMED, InputResult_HAS_COMMIT,
    InputResult_HAS_PREEDIT, InputResult_LANGUAGE_CHANGED, InputResult_NOT_READY, ModifierState,
    ModifierState_ALT, ModifierState_CONTROL, ModifierState_SHIFT, ModifierState_SUPER,
    KIME_API_VERSION,
};

pub fn check_api_version() -> bool {
    unsafe { ffi::kime_api_version() == ffi::KIME_API_VERSION }
}

pub struct InputEngine {
    engine: *mut ffi::InputEngine,
}

impl InputEngine {
    pub fn new(config: &Config) -> Self {
        Self {
            engine: unsafe { ffi::kime_engine_new(config.config) },
        }
    }

    pub fn update_layout_state(&self) {
        unsafe { ffi::kime_engine_update_layout_state(self.engine) }
    }

    pub fn end_ready(&mut self) -> InputResult {
        unsafe { ffi::kime_engine_end_ready(self.engine) }
    }

    pub fn check_ready(&self) -> bool {
        unsafe { ffi::kime_engine_check_ready(self.engine) }
    }

    pub fn set_input_category(&mut self, category: InputCategory) {
        unsafe { ffi::kime_engine_set_input_category(self.engine, category) };
    }

    pub fn press_key(
        &mut self,
        config: &Config,
        hardware_code: u16,
        state: ModifierState,
    ) -> InputResult {
        unsafe { ffi::kime_engine_press_key(self.engine, config.config, hardware_code, state) }
    }

    pub fn preedit_str(&mut self) -> &str {
        unsafe {
            let s = ffi::kime_engine_preedit_str(self.engine);
            core::str::from_utf8_unchecked(core::slice::from_raw_parts(s.ptr, s.len))
        }
    }

    pub fn commit_str(&self) -> &str {
        unsafe {
            let s = ffi::kime_engine_commit_str(self.engine);
            core::str::from_utf8_unchecked(core::slice::from_raw_parts(s.ptr, s.len))
        }
    }

    pub fn clear_commit(&mut self) {
        unsafe {
            ffi::kime_engine_clear_commit(self.engine);
        }
    }

    pub fn clear_preedit(&mut self) {
        unsafe {
            ffi::kime_engine_clear_preedit(self.engine);
        }
    }

    pub fn remove_preedit(&mut self) {
        unsafe {
            ffi::kime_engine_remove_preedit(self.engine);
        }
    }

    pub fn reset(&mut self) {
        unsafe {
            ffi::kime_engine_reset(self.engine);
        }
    }
}

impl Drop for InputEngine {
    fn drop(&mut self) {
        unsafe {
            ffi::kime_engine_delete(self.engine);
        }
    }
}

pub struct Config {
    config: *mut ffi::Config,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            config: unsafe { ffi::kime_config_default() },
        }
    }
}

impl Config {
    #[cfg(unix)]
    pub fn load() -> Self {
        Self {
            config: unsafe { ffi::kime_config_load() },
        }
    }

    pub fn xim_font(&self) -> (&str, f64) {
        unsafe {
            let font = ffi::kime_config_xim_preedit_font(self.config);

            (
                core::str::from_utf8_unchecked(core::slice::from_raw_parts(
                    font.name.ptr,
                    font.name.len,
                )),
                font.size,
            )
        }
    }
}

impl Drop for Config {
    fn drop(&mut self) {
        unsafe {
            ffi::kime_config_delete(self.config);
        }
    }
}

pub struct DaemonConfig {
    config: *mut ffi::DaemonConfig,
}

impl DaemonConfig {
    #[cfg(unix)]
    pub fn load() -> Self {
        Self {
            config: unsafe { ffi::kime_daemon_config_load() },
        }
    }

    pub fn modules(&self) -> EnumSet<DaemonModule> {
        EnumSet::from_u32(unsafe { ffi::kime_daemon_config_modules(self.config) })
    }
}

impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            config: unsafe { ffi::kime_daemon_config_default() },
        }
    }
}

impl Drop for DaemonConfig {
    fn drop(&mut self) {
        unsafe {
            ffi::kime_daemon_config_delete(self.config);
        }
    }
}

pub struct IndicatorConfig {
    config: *mut ffi::IndicatorConfig,
}

impl IndicatorConfig {
    #[cfg(unix)]
    pub fn load() -> Self {
        Self {
            config: unsafe { ffi::kime_indicator_config_load() },
        }
    }

    pub fn icon_color(&self) -> IconColor {
        unsafe { ffi::kime_indicator_config_icon_color(self.config) }
    }
}

impl Default for IndicatorConfig {
    fn default() -> Self {
        Self {
            config: unsafe { ffi::kime_indicator_config_default() },
        }
    }
}

impl Drop for IndicatorConfig {
    fn drop(&mut self) {
        unsafe {
            ffi::kime_indicator_config_delete(self.config);
        }
    }
}

pub struct LogConfig {
    config: *mut ffi::LogConfig,
}

impl LogConfig {
    #[cfg(unix)]
    pub fn load() -> Self {
        Self {
            config: unsafe { ffi::kime_log_config_load() },
        }
    }

    pub fn global_level(&self) -> &'static str {
        unsafe {
            let s = ffi::kime_log_config_global_level(self.config);
            core::str::from_utf8_unchecked(core::slice::from_raw_parts(s.ptr, s.len))
        }
    }
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            config: unsafe { ffi::kime_log_config_default() },
        }
    }
}

impl Drop for LogConfig {
    fn drop(&mut self) {
        unsafe {
            ffi::kime_log_config_delete(self.config);
        }
    }
}
