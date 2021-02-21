/* automatically generated by rust-bindgen 0.56.0 */

pub type __uint8_t = ::std::os::raw::c_uchar;
pub type __uint16_t = ::std::os::raw::c_ushort;
pub type __uint32_t = ::std::os::raw::c_uint;
pub const KIME_API_VERSION: usize = 2;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Config {
    _unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct InputEngine {
    _unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct RustStr {
    pub ptr: *const u8,
    pub len: usize,
}
#[test]
fn bindgen_test_layout_RustStr() {
    assert_eq!(
        ::std::mem::size_of::<RustStr>(),
        16usize,
        concat!("Size of: ", stringify!(RustStr))
    );
    assert_eq!(
        ::std::mem::align_of::<RustStr>(),
        8usize,
        concat!("Alignment of ", stringify!(RustStr))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<RustStr>())).ptr as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(RustStr),
            "::",
            stringify!(ptr)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<RustStr>())).len as *const _ as usize },
        8usize,
        concat!(
            "Offset of field: ",
            stringify!(RustStr),
            "::",
            stringify!(len)
        )
    );
}
pub type InputResult = u32;
pub const InputResult_CONSUMED: InputResult = 1;
pub const InputResult_LANGUAGE_CHANGED: InputResult = 2;
pub const InputResult_HAS_PREEDIT: InputResult = 4;
pub const InputResult_NEED_RESET: InputResult = 8;
pub const InputResult_NEED_FLUSH: InputResult = 16;
pub type ModifierState = u32;
pub const ModifierState_CONTROL: ModifierState = 1;
pub const ModifierState_SUPER: ModifierState = 2;
pub const ModifierState_SHIFT: ModifierState = 4;
pub const ModifierState_ALT: ModifierState = 8;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct XimPreeditFont {
    pub name: RustStr,
    pub size: f64,
}
#[test]
fn bindgen_test_layout_XimPreeditFont() {
    assert_eq!(
        ::std::mem::size_of::<XimPreeditFont>(),
        24usize,
        concat!("Size of: ", stringify!(XimPreeditFont))
    );
    assert_eq!(
        ::std::mem::align_of::<XimPreeditFont>(),
        8usize,
        concat!("Alignment of ", stringify!(XimPreeditFont))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<XimPreeditFont>())).name as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(XimPreeditFont),
            "::",
            stringify!(name)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<XimPreeditFont>())).size as *const _ as usize },
        16usize,
        concat!(
            "Offset of field: ",
            stringify!(XimPreeditFont),
            "::",
            stringify!(size)
        )
    );
}
extern "C" {
    #[doc = " Return API version"]
    pub fn kime_api_version() -> usize;
}
extern "C" {
    #[doc = " Create new engine"]
    pub fn kime_engine_new(config: *const Config) -> *mut InputEngine;
}
extern "C" {
    #[doc = " Set hangul enable state"]
    pub fn kime_engine_set_hangul_enable(engine: *mut InputEngine, mode: bool);
}
extern "C" {
    #[doc = " Delete engine"]
    #[doc = ""]
    #[doc = " # Safety"]
    #[doc = ""]
    #[doc = " engine must be created by `kime_engine_new` and never call delete more than once"]
    pub fn kime_engine_delete(engine: *mut InputEngine);
}
extern "C" {
    #[doc = " Update hangul state"]
    pub fn kime_engine_update_hangul_state(engine: *mut InputEngine);
}
extern "C" {
    #[doc = " Get commit string of engine"]
    #[doc = ""]
    #[doc = " ## Return"]
    #[doc = ""]
    #[doc = " valid utf8 string"]
    pub fn kime_engine_commit_str(engine: *mut InputEngine) -> RustStr;
}
extern "C" {
    #[doc = " Get preedit string of engine"]
    #[doc = ""]
    #[doc = " ## Return"]
    #[doc = ""]
    #[doc = " valid utf8 string"]
    pub fn kime_engine_preedit_str(engine: *mut InputEngine) -> RustStr;
}
extern "C" {
    #[doc = " Flush commit_str"]
    pub fn kime_engine_flush(engine: *mut InputEngine);
}
extern "C" {
    #[doc = " Clear preedit state and append to commit_str"]
    pub fn kime_engine_clear_preedit(engine: *mut InputEngine);
}
extern "C" {
    #[doc = " Reset preedit state then returm commit char"]
    pub fn kime_engine_reset(engine: *mut InputEngine);
}
extern "C" {
    #[doc = " Press key when modifier state"]
    #[doc = ""]
    #[doc = " ## Return"]
    #[doc = ""]
    #[doc = " input result"]
    pub fn kime_engine_press_key(
        engine: *mut InputEngine,
        config: *const Config,
        hardware_code: u16,
        state: ModifierState,
    ) -> InputResult;
}
extern "C" {
    #[doc = " Load config from local file"]
    pub fn kime_config_load() -> *mut Config;
}
extern "C" {
    #[doc = " Create default config note that this function will not read config file"]
    pub fn kime_config_default() -> *mut Config;
}
extern "C" {
    #[doc = " Delete config"]
    pub fn kime_config_delete(config: *mut Config);
}
extern "C" {
    #[doc = " Get xim_preedit_font config"]
    #[doc = " name only valid while config is live"]
    #[doc = ""]
    #[doc = " ## Return"]
    #[doc = ""]
    #[doc = " utf-8 string when len"]
    pub fn kime_config_xim_preedit_font(config: *const Config) -> XimPreeditFont;
}
