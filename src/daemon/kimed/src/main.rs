use anyhow::Result;
use gobject_sys::g_signal_connect_data;
use kimed_types::{bincode, ClientMessage, GetGlobalHangulStateReply};
use libappindicator_sys::{AppIndicator, AppIndicatorStatus_APP_INDICATOR_STATUS_ACTIVE};
use std::ffi::CString;
use std::fs::File;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;
use std::ptr;
use std::sync::atomic::{AtomicBool, Ordering::Relaxed};
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

static GLOBAL_HANGUL_STATE: AtomicBool = AtomicBool::new(false);

fn serve_client(mut stream: UnixStream, indicator_tx: glib::SyncSender<bool>) -> Result<()> {
    loop {
        match bincode::deserialize_from(&mut stream)? {
            ClientMessage::GetGlobalHangulState => {
                bincode::serialize_into(
                    &mut stream,
                    &GetGlobalHangulStateReply {
                        state: GLOBAL_HANGUL_STATE.load(Relaxed),
                    },
                )?;
            }
            ClientMessage::UpdateHangulState(state) => {
                GLOBAL_HANGUL_STATE.store(state, Relaxed);

                if indicator_tx.send(state).is_err() {
                    break Ok(());
                }
            }
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

    std::thread::spawn(move || {
        let path = std::path::Path::new("/tmp/kimed.sock");
        if path.exists() {
            std::fs::remove_file(path).ok();
        }

        let server = UnixListener::bind(path).unwrap();

        loop {
            let (stream, _addr) = server.accept().expect("Accept");
            log::info!("Connect client");
            let tx = indicator_tx.clone();
            std::thread::spawn(move || {
                if let Err(err) = serve_client(stream, tx) {
                    log::error!("Client Error: {}", err);
                }
            });
        }
    });

    unsafe {
        gtk_sys::gtk_main();
    }

    Ok(())
}

#[derive(StructOpt)]
#[structopt(about = "kime daemon")]
struct Opts {
    #[structopt(long, short, help = "Show daemon version")]
    version: bool,
    #[structopt(long, help = "Log more messages")]
    verbose: bool,
    #[structopt(long, help = "Run as normal process")]
    not_daemon: bool,
}

fn main() {
    let opt = Opts::from_args();

    if opt.version {
        kime_version::print_version!();
        return;
    }

    if !opt.not_daemon {
        let daemonize = daemonize::Daemonize::new()
            .pid_file("/tmp/kimed.pid")
            .working_directory("/tmp")
            .stdout(File::create("/tmp/kimed.out").unwrap())
            .stderr(File::create("/tmp/kimed.err").unwrap());

        if let Err(err) = daemonize.start() {
            eprintln!("Daemonize Error: {}", err);
            return;
        }
    }

    simplelog::SimpleLogger::init(
        if cfg!(debug_assertions) || opt.verbose {
            log::LevelFilter::Trace
        } else {
            log::LevelFilter::Info
        },
        simplelog::ConfigBuilder::new().build(),
    )
    .ok();
    log::info!("Start daemon");
    match daemon_main() {
        Ok(_) => {}
        Err(err) => {
            log::error!("Error: {}", err);
        }
    }
}
