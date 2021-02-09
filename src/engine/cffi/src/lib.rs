use std::char::from_u32_unchecked;
use std::mem::MaybeUninit;

#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
mod ffi;

#[link(name = "kime_engine", kind = "dylib")]
extern "C" {}

pub use ffi::{
    InputResultType, ModifierState, ModifierState_ALT, ModifierState_CONTROL, ModifierState_SHIFT,
    ModifierState_SUPER,
};

#[derive(Clone, Copy, Debug)]
pub struct InputResult {
    pub ty: InputResultType,
    pub hangul_changed: bool,
    pub char1: char,
    pub char2: char,
}

pub struct InputEngine {
    engine: *mut ffi::InputEngine,
}

impl InputEngine {
    pub fn new() -> Self {
        Self {
            engine: unsafe { ffi::kime_engine_new() },
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
        let ret =
            unsafe { ffi::kime_engine_press_key(self.engine, config.config, hardware_code, state) };

        unsafe {
            InputResult {
                ty: ret.ty,
                hangul_changed: ret.hangul_changed,
                char1: from_u32_unchecked(ret.char1),
                char2: from_u32_unchecked(ret.char2),
            }
        }
    }

    /// `NULL` mean empty
    pub fn preedit_char(&self) -> char {
        unsafe { from_u32_unchecked(ffi::kime_engine_preedit_char(self.engine)) }
    }

    /// `NULL` mean empty
    pub fn reset(&mut self) -> char {
        unsafe { from_u32_unchecked(ffi::kime_engine_reset(self.engine)) }
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
            let mut ptr = MaybeUninit::uninit();
            let mut len = MaybeUninit::uninit();
            let mut size = MaybeUninit::uninit();
            ffi::kime_config_xim_preedit_font(
                self.config,
                ptr.as_mut_ptr(),
                len.as_mut_ptr(),
                size.as_mut_ptr(),
            );

            (
                std::str::from_utf8_unchecked(std::slice::from_raw_parts(
                    ptr.assume_init(),
                    len.assume_init(),
                )),
                size.assume_init(),
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
