#![allow(clippy::missing_safety_doc)]

pub use kime_engine_core::{Config, InputCategory, InputEngine, InputResult, ModifierState};
#[cfg(unix)]
use nix::{
    fcntl::OFlag,
    sys::{
        mman::{mmap, munmap, shm_open, shm_unlink, MapFlags, ProtFlags},
        stat::Mode,
    },
    unistd::ftruncate,
};
use std::{mem, num::NonZeroU32, ptr};

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

#[cfg(unix)]
const KIME_SHM_NAME: &str = "kime-config";
#[cfg(unix)]
const KIME_SHM_VERSION: Option<NonZeroU32> = NonZeroU32::new(1);
#[cfg(unix)]
const KIME_CONFIG_SIZE: usize = mem::size_of::<Config>();

pub const KIME_API_VERSION: usize = 5;

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

#[cfg(unix)]
fn kime_config_shm(config_factory: &dyn Fn() -> Config) -> nix::Result<*const Config> {
    loop {
        match shm_open(
            KIME_SHM_NAME,
            OFlag::O_CREAT | OFlag::O_EXCL | OFlag::O_RDWR,
            Mode::S_IRUSR | Mode::S_IWUSR | Mode::S_IRGRP | Mode::S_IROTH,
        ) {
            Ok(shm_fd) => unsafe {
                ftruncate(shm_fd, KIME_CONFIG_SIZE as _)?;
                let config = mmap(
                    ptr::null_mut(),
                    KIME_CONFIG_SIZE,
                    ProtFlags::PROT_READ | ProtFlags::PROT_WRITE,
                    MapFlags::MAP_SHARED,
                    shm_fd,
                    0,
                )?
                .cast::<Config>();
                config.write(config_factory());
                (*config).shm_version = KIME_SHM_VERSION;
                break Ok(config);
            },
            // already exists
            Err(nix::Error::Sys(nix::errno::Errno::EEXIST)) => {
                let shm_fd = shm_open(KIME_SHM_NAME, OFlag::O_RDONLY, Mode::empty())?;
                unsafe {
                    let config = mmap(
                        ptr::null_mut(),
                        KIME_CONFIG_SIZE,
                        ProtFlags::PROT_READ,
                        MapFlags::MAP_SHARED,
                        shm_fd,
                        0,
                    )?
                    .cast::<Config>();

                    // unmatched version find remove and retry
                    if (*config).shm_version != KIME_SHM_VERSION {
                        munmap(config.cast(), KIME_CONFIG_SIZE)?;
                        shm_unlink(KIME_SHM_NAME)?;
                        continue;
                    }

                    break Ok(config.cast());
                }
            }
            Err(err) => break Err(err),
        }
    }
}

/// Load config from local file
/// If loading failed, it return NULL!
#[cfg(unix)]
#[no_mangle]
pub extern "C" fn kime_config_load() -> *const Config {
    let factory = || Config::load_from_config_dir().unwrap_or_default();
    kime_config_shm(&factory).unwrap_or_else(|_| Box::into_raw(Box::new(factory())))
}

/// Delete config
#[no_mangle]
pub unsafe extern "C" fn kime_config_delete(config: *const Config) {
    if (*config).shm_version.is_some() {
        #[cfg(unix)]
        munmap(
            config as *const std::ffi::c_void as *mut _,
            KIME_CONFIG_SIZE,
        )
        .ok();
    } else {
        Box::from_raw(config as *mut Config);
    }
}

/// Create default config note that this function will not read config file and shm
#[no_mangle]
pub extern "C" fn kime_config_default() -> *mut Config {
    Box::into_raw(Box::new(Config::default()))
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
