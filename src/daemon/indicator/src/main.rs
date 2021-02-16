use anyhow::Result;
use gio::{prelude::*, FileExt, FileMonitorEvent};
use gobject_sys::g_signal_connect_data;
use libappindicator_sys::{AppIndicator, AppIndicatorStatus_APP_INDICATOR_STATUS_ACTIVE};
use std::ffi::CString;
use std::path::Path;
use std::ptr;
use structopt::StructOpt;

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

    indicator.disable_hangul();

    let (indicator_tx, indicator_rx) =
        glib::MainContext::sync_channel(glib::PRIORITY_DEFAULT_IDLE, 10);

    let ctx = glib::MainContext::ref_thread_default();

    assert!(ctx.acquire());

    indicator_rx.attach(Some(&ctx), move |msg| {
        if msg {
            indicator.enable_hangul();
        } else {
            indicator.disable_hangul();
        }

        glib::Continue(true)
    });

    ctx.release();

    let cancellable: Option<&gio::Cancellable> = None;
    let path = Path::new("/tmp/kime_hangul_state");
    let file = gio::File::new_for_path(path);
    let monitor = file
        .monitor_file(gio::FileMonitorFlags::WATCH_MOVES, cancellable)
        .expect("Create Monitor");

    monitor.connect_changed(move |_m, f, _, e| match e {
        FileMonitorEvent::Created | FileMonitorEvent::Changed => {
            let mut buf = [0; 1];
            let read = f.read(cancellable).unwrap();
            let len = read.read(&mut buf[..], cancellable).unwrap();

            if len > 0 {
                if buf[0] == b'1' {
                    indicator_tx.send(true).ok();
                } else {
                    indicator_tx.send(false).ok();
                }
            }
        }
        _ => {}
    });

    unsafe {
        gtk_sys::gtk_main();
    }

    Ok(())
}

#[derive(StructOpt)]
#[structopt(about = "kime-indicator")]
struct Opts {
    #[structopt(long, short, help = "Show indicator version")]
    version: bool,
}

fn main() {
    let opt = Opts::from_args();

    if opt.version {
        kime_version::print_version!();
        return;
    }

    kime_log::enable_logger_with_env();
    log::info!("Start indicator");
    match daemon_main() {
        Ok(_) => {}
        Err(err) => {
            log::error!("Error: {}", err);
        }
    }
}
