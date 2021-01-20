use std::char::from_u32_unchecked;
use std::mem::MaybeUninit;

#[link(name = "kime_engine")]
extern "C" {
}

#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
mod ffi;

pub use ffi::{InputResultType, ModifierState};

pub const MODIFIER_CONTROL: ModifierState = 1;
pub const MODIFIER_SUPER: ModifierState = 2;
pub const MODIFIER_SHIFT: ModifierState = 4;

#[derive(Clone, Copy, Debug)]
pub struct InputResult {
    pub ty: InputResultType,
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
                char1: from_u32_unchecked(ret.char1),
                char2: from_u32_unchecked(ret.char2),
            }
        }
    }

    pub fn preedit_char(&self) -> Option<char> {
        unsafe {
            match ffi::kime_engine_preedit_char(self.engine) {
                0 => None,
                n => Some(from_u32_unchecked(n)),
            }
        }
    }

    pub fn reset(&mut self) -> Option<char> {
        unsafe {
            match ffi::kime_engine_reset(self.engine) {
                0 => None,
                n => Some(from_u32_unchecked(n)),
            }
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

impl Config {
    pub fn new() -> Self {
        Self {
            config: unsafe { ffi::kime_config_load() },
        }
    }

    pub fn xim_font_name(&self) -> &str {
        unsafe {
            let mut ptr = MaybeUninit::uninit();
            let mut len = MaybeUninit::uninit();
            ffi::kime_config_xim_preedit_font(self.config, ptr.as_mut_ptr(), len.as_mut_ptr());
            std::str::from_utf8_unchecked(std::slice::from_raw_parts(
                ptr.assume_init(),
                len.assume_init(),
            ))
        }
    }

    pub fn gtk_commit_english(&self) -> bool {
        unsafe { ffi::kime_config_gtk_commit_english(self.config) != 0 }
    }
}

impl Drop for Config {
    fn drop(&mut self) {
        unsafe {
            ffi::kime_config_delete(self.config);
        }
    }
}
