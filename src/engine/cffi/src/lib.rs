#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
mod ffi;

#[link(name = "kime_engine", kind = "dylib")]
extern "C" {}

pub use ffi::{
    InputResult, InputResult_CONSUMED, InputResult_NEED_FLUSH, InputResult_HAS_PREEDIT,
    InputResult_LANGUAGE_CHANGED, InputResult_NEED_UPDATE, ModifierState, ModifierState_ALT,
    ModifierState_CONTROL, ModifierState_SHIFT, ModifierState_SUPER,
};

pub fn check_api_version() -> bool {
    unsafe { ffi::kime_api_version() == 1 }
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

    pub fn update_hangul_state(&self) {
        unsafe { ffi::kime_engine_update_hangul_state(self.engine) }
    }

    pub fn set_hangul_enable(&mut self, mode: bool) {
        unsafe { ffi::kime_engine_set_hangul_enable(self.engine, mode) };
    }

    pub fn press_key(
        &mut self,
        config: &Config,
        hardware_code: u16,
        state: ModifierState,
    ) -> InputResult {
        unsafe { ffi::kime_engine_press_key(self.engine, config.config, hardware_code, state) }
    }

    pub fn preedit_str(&self) -> &str {
        unsafe {
            let s = ffi::kime_engine_preedit_str(self.engine);
            std::str::from_utf8_unchecked(std::slice::from_raw_parts(s.ptr, s.len))
        }
    }

    pub fn commit_str(&self) -> &str {
        unsafe {
            let s = ffi::kime_engine_commit_str(self.engine);
            std::str::from_utf8_unchecked(std::slice::from_raw_parts(s.ptr, s.len))
        }
    }

    pub fn flush(&mut self) {
        unsafe {
            ffi::kime_engine_flush(self.engine);
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
    pub fn load() -> Self {
        Self {
            config: unsafe { ffi::kime_config_load() },
        }
    }

    pub fn xim_font(&self) -> (&str, f64) {
        unsafe {
            let font = ffi::kime_config_xim_preedit_font(self.config);

            (
                std::str::from_utf8_unchecked(std::slice::from_raw_parts(
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
