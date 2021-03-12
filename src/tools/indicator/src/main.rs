use anyhow::Result;
use gio::{prelude::*, FileExt, FileMonitorEvent};
use gobject_sys::g_signal_connect_data;
use libappindicator_sys::{AppIndicator, AppIndicatorStatus_APP_INDICATOR_STATUS_ACTIVE};
use std::path::Path;
use std::ptr;
use std::{env, path::PathBuf};
use std::{ffi::CString, io};

macro_rules! cs {
    ($ex:expr) => {
        concat!($ex, "\0").as_ptr().cast()
    };
}

#[derive(Clone, Copy)]
enum Lang {
    Eng,
    Han,
}

#[derive(Clone, Copy)]
enum IconColor {
    Black,
    White,
}

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
            let icon_dirs = xdg::BaseDirectories::with_prefix("kime/icons").unwrap();
            let indicator = libappindicator_sys::app_indicator_new(
                cs!("kime"),
                cs!(""),
                libappindicator_sys::AppIndicatorCategory_APP_INDICATOR_CATEGORY_APPLICATION_STATUS,
            );

            let icon = icon_dirs
                .find_data_file("kime-han-white-64x64.png")
                .expect("Can't find image");
            set_icon_path(indicator, icon.parent().unwrap());

            gtk_sys::gtk_widget_show_all(m);
            libappindicator_sys::app_indicator_set_menu(indicator, m.cast());
            libappindicator_sys::app_indicator_set_status(
                indicator,
                AppIndicatorStatus_APP_INDICATOR_STATUS_ACTIVE,
            );

            Self { indicator }
        }
    }

    pub fn update(&self, color: IconColor, lang: Lang) {
        unsafe {
            match lang {
                Lang::Eng => {
                    libappindicator_sys::app_indicator_set_icon(
                        self.indicator,
                        match color {
                            IconColor::Black => cs!("kime-eng-black-64x64"),
                            IconColor::White => cs!("kime-eng-white-64x64"),
                        },
                    );
                }
                Lang::Han => {
                    libappindicator_sys::app_indicator_set_icon(
                        self.indicator,
                        match color {
                            IconColor::Black => cs!("kime-han-black-64x64"),
                            IconColor::White => cs!("kime-han-white-64x64"),
                        },
                    );
                }
            }
        }
    }
}

fn indicator_server(file_path: &Path) -> Result<()> {
    unsafe {
        gtk_sys::gtk_init(ptr::null_mut(), ptr::null_mut());
    }

    let indicator = Indicator::new();
    indicator.update(IconColor::Black, Lang::Eng);

    let cancellable: Option<&gio::Cancellable> = None;
    let file = gio::File::new_for_path(file_path);
    let monitor = file
        .monitor_file(gio::FileMonitorFlags::WATCH_MOVES, cancellable)
        .expect("Create Monitor");

    monitor.connect_changed(move |_m, f, _, e| match e {
        FileMonitorEvent::Created | FileMonitorEvent::Changed => {
            let mut buf = [0; 2];
            let read = f.read(cancellable).unwrap();
            let len = read.read_all(&mut buf[..], cancellable).unwrap().0;

            if len == 2 {
                let color = match buf[0] {
                    0 => IconColor::Black,
                    _ => IconColor::White,
                };
                let lang = match buf[1] {
                    0 => Lang::Eng,
                    _ => Lang::Han,
                };

                indicator.update(color, lang);
            }
        }
        _ => {}
    });

    unsafe {
        gtk_sys::gtk_main();
    }

    Ok(())
}

fn client_send(file_path: &Path, color: IconColor, lang: Lang) -> io::Result<()> {
    let mut msg = [0; 2];

    msg[0] = match color {
        IconColor::Black => 0,
        IconColor::White => 1,
    };

    msg[1] = match lang {
        Lang::Eng => 0,
        Lang::Han => 1,
    };

    std::fs::write(file_path, &msg)
}

fn main() {
    let mut args = kime_version::cli_boilerplate!(
        "--dark: show dark icon (default)",
        "--white: show white icon",
        "--english: set english (default)",
        "--hangul: set hangul",
    );

    let mut color = IconColor::Black;

    if args.contains("--white") {
        color = IconColor::White;
    }

    let mut lang = Lang::Eng;

    if args.contains("--hangul") {
        lang = Lang::Han;
    }

    let run_dir = PathBuf::from(env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".into()));
    let pid_path = run_dir.join("kime-indicator.pid");
    let file_path = run_dir.join("kime-indicator.state");
    let file_path_inner = file_path.clone();

    let daemonize = daemonize::Daemonize::new()
        .pid_file(pid_path)
        .exit_action(move || {
            client_send(&file_path_inner, color, lang).ok();
        });

    match daemonize.start() {
        Ok(_) => match indicator_server(&file_path) {
            Ok(_) => {}
            Err(err) => {
                log::error!("Error: {}", err);
            }
        },
        // Already running
        Err(daemonize::DaemonizeError::LockPidfile(_)) => {
            client_send(&file_path, color, lang).ok();
        }
        Err(err) => {
            log::error!("Start failed: {}", err);
        }
    }
}
