use gio::prelude::*;
use gtk::prelude::*;
use libappindicator::{AppIndicator, AppIndicatorStatus};
use libc::mkfifo;
use std::fs::File;
use std::io::{self, Read, Result};

macro_rules! cs {
    ($ex:expr) => {
        concat!($ex, "\0").as_ptr().cast()
    };
}

const HAN_ICON: &str = "kime-han-64x64.png";
const ENG_ICON: &str = "kime-eng-64x64.png";

struct Indicator {
    indicator: AppIndicator,
}

impl Indicator {
    pub fn new() -> Self {
        let mut m = gtk::Menu::new();
        let mi = gtk::CheckMenuItem::with_label("Exit");
        mi.connect_activate(|_| {
            gtk::main_quit();
        });
        m.append(&mi);
        let icon_dirs = xdg::BaseDirectories::with_profile("kime", "icons").unwrap();
        let mut indicator = AppIndicator::new("kime", "");
        let han = icon_dirs.find_data_file(HAN_ICON).unwrap();
        let eng = icon_dirs.find_data_file(ENG_ICON).unwrap();
        indicator.set_icon_theme_path(han.parent().unwrap().to_str().unwrap());
        if han != eng {
            indicator.set_icon_theme_path(eng.parent().unwrap().to_str().unwrap());
        }
        indicator.set_icon_full("kime-han-64x64", "icon");
        indicator.set_status(AppIndicatorStatus::Active);
        indicator.set_menu(&mut m);
        m.show_all();

        Self { indicator }
    }

    pub fn enable_hangul(&mut self) {
        println!("enable");
        self.indicator.set_icon_full("kime-han-64x64", "icon");
    }

    pub fn disable_hangul(&mut self) {
        println!("disable");
        self.indicator.set_icon_full("kime-eng-64x64", "icon");
    }
}

fn daemon_main() -> Result<()> {
    gtk::init().unwrap();

    let mut indicator = Indicator::new();

    let path = std::path::Path::new("/tmp/kimed_hangul_state");

    if path.exists() {
        std::fs::remove_file(path)?;
    }

    if unsafe { mkfifo(cs!("/tmp/kimed_hangul_state"), 0o644) } != 0 {
        eprintln!("Failed mkfifo");
        return Err(io::Error::last_os_error());
    }

    let pipe = gio::File::new_for_path(path);

    let c = glib::MainContext::default();
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

    gtk::main();

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
        daemon_main().unwrap();
    }
}
