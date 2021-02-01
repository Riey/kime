use gio::{prelude::InputStreamExtManual, FileExt};
use gobject_sys::g_signal_connect_data;
use libappindicator_sys::{AppIndicator, AppIndicatorStatus_APP_INDICATOR_STATUS_ACTIVE};
use libc::mkfifo;
use std::ffi::CString;
use std::fs::File;
use std::io::{self, Read, Result};
use std::path::Path;
use std::ptr;

macro_rules! cs {
    ($ex:expr) => {
        concat!($ex, "\0").as_ptr().cast()
    };
}

const HAN_ICON: &str = "kime-han-64x64.png";
const ENG_ICON: &str = "kime-eng-64x64.png";

struct Indicator {
    indicator: *mut AppIndicator,
}

impl Indicator {
    pub fn new() -> Self {
        unsafe fn set_icon_path(indicator: *mut AppIndicator, path: &Path) {
            let s = path.to_str().unwrap();
            let s = CString::new(s).unwrap();
            libappindicator_sys::app_indicator_set_icon_theme_path(indicator, s.as_ptr());
        }

        unsafe {
            let m = gtk_sys::gtk_menu_new();
            let mi = gtk_sys::gtk_check_menu_item_new_with_label(cs!("Exit"));
            unsafe extern "C" fn exit() {
                gtk_sys::gtk_main_quit();
            }
            g_signal_connect_data(
                mi.cast(),
                cs!("activate"),
                Some(exit),
                ptr::null_mut(),
                None,
                0,
            );
            gtk_sys::gtk_menu_shell_append(m.cast(), mi.cast());
            let icon_dirs = xdg::BaseDirectories::with_profile("kime", "icons").unwrap();
            let indicator = libappindicator_sys::app_indicator_new(
                cs!("kime"),
                cs!(""),
                libappindicator_sys::AppIndicatorCategory_APP_INDICATOR_CATEGORY_APPLICATION_STATUS,
            );
            let han = icon_dirs.find_data_file(HAN_ICON).unwrap();
            let eng = icon_dirs.find_data_file(ENG_ICON).unwrap();
            set_icon_path(indicator, han.parent().unwrap());
            set_icon_path(indicator, eng.parent().unwrap());
            libappindicator_sys::app_indicator_set_status(
                indicator,
                AppIndicatorStatus_APP_INDICATOR_STATUS_ACTIVE,
            );
            libappindicator_sys::app_indicator_set_menu(indicator, m.cast());
            gtk_sys::gtk_widget_show_all(m);
            Self { indicator }
        }
    }

    pub fn enable_hangul(&mut self) {
        unsafe {
            libappindicator_sys::app_indicator_set_icon_full(
                self.indicator,
                cs!("kime-han-64x64"),
                cs!("icon"),
            );
        }
    }

    pub fn disable_hangul(&mut self) {
        unsafe {
            libappindicator_sys::app_indicator_set_icon_full(
                self.indicator,
                cs!("kime-eng-64x64"),
                cs!("icon"),
            );
        }
    }
}

fn daemon_main() -> Result<()> {
    unsafe {
        gtk_sys::gtk_init(ptr::null_mut(), ptr::null_mut());
    }

    let mut indicator = Indicator::new();

    indicator.enable_hangul();

    let path = std::path::Path::new("/tmp/kimed_hangul_state");

    if path.exists() {
        std::fs::remove_file(path)?;
    }

    if unsafe { mkfifo(cs!("/tmp/kimed_hangul_state"), 0o644) } != 0 {
        log::error!("Failed mkfifo");
        return Err(io::Error::last_os_error());
    }

    let pipe = gio::File::new_for_path(path);
    let c = glib::MainContext::default();
    assert!(c.acquire());
    c.spawn_local(async move {
        loop {
            let ret: gio::FileInputStream = pipe
                .read_async_future(glib::PRIORITY_DEFAULT_IDLE)
                .await
                .unwrap();
            let mut buf = [0; 1024];
            let len = ret.into_read().read(&mut buf).unwrap();

            if len < 1 {
                continue;
            }

            if buf[0] == b'0' {
                indicator.disable_hangul();
            } else {
                indicator.enable_hangul();
            }
        }
    });
    c.release();

    unsafe {
        gtk_sys::gtk_main();
    }

    Ok(())
}

fn main() {
    let daemonize = daemonize::Daemonize::new()
        .pid_file("/tmp/kimed.pid")
        .working_directory("/tmp")
        .stdout(File::create("/tmp/kimed.out").unwrap())
        .stderr(File::create("/tmp/kimed.err").unwrap());

    if let Err(err) = daemonize.start() {
        eprintln!("Daemonize Error: {}", err);
    } else {
        syslog::init_unix(syslog::Facility::LOG_DAEMON, log::LevelFilter::Trace).expect("Init syslog");
        match daemon_main() {
            Ok(_) => {},
            Err(err) => {
                log::error!("Error: {}", err);
            }
        }
    }
}
