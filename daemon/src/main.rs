use gtk::prelude::*;
use libappindicator::{AppIndicator, AppIndicatorStatus};
use libc::mkfifo;
use std::fs::{File, OpenOptions};
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Message {
    EnableHangul,
    DisableHangul,
}

fn daemon_main() -> Result<()> {
    gtk::init().unwrap();
    let mut indicator = Indicator::new();

    let path = std::path::Path::new("/tmp/kime_hangul_state");

    if path.exists() {
        std::fs::remove_file(path)?;
    }

    // let pipe = gio::File::new_for_path(path);

    // if unsafe { mkfifo(cs!("/tmp/kime_hangul_state"), 0o644) } != 0 {
    //     eprintln!("Failed mkfifo");
    //     return Err(io::Error::last_os_error());
    // }

    // loop {
    //     let mut pipe = OpenOptions::new().read(true).open(path).unwrap();

    //     let mut buf = [0; 1];
    //     pipe.read_exact(&mut buf).unwrap();

    //     if buf[0] == b'0' {
    //         indicator.disable_hangul();
    //     } else {
    //         indicator.enable_hangul();
    //     }
    // }

    gtk::main();

    Ok(())
}

fn main() {
    let daemonize = daemonize::Daemonize::new()
        .pid_file("/tmp/kime-daemon.pid")
        .working_directory("/tmp")
        .stdout(File::create("/tmp/kime-daemon.out").unwrap())
        .stderr(File::create("/tmp/kime-daemon.err").unwrap());

    if let Err(err) = daemonize.start() {
        eprintln!("Daemonize Error: {}", err);
    } else {
        daemon_main().unwrap();
    }
}
