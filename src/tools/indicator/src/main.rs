use anyhow::Result;
use ksni::menu::*;
use std::net::Shutdown;
use std::os::unix::net::UnixListener;
use std::path::Path;
use std::{
    io::{Read, Write},
    time::Duration,
};

#[derive(Clone, Copy, Debug)]
enum InputCategory {
    Latin,
    Hangul,
}

#[derive(Clone, Copy, Debug)]
enum IconColor {
    Black,
    White,
}

struct KimeTray {
    icon_name: &'static str,
}

impl ksni::Tray for KimeTray {
    fn icon_name(&self) -> String {
        self.icon_name.into()
    }

    fn title(&self) -> String {
        "kime".into()
    }
    fn attention_icon_name(&self) -> String {
        self.icon_name.into()
    }
    fn menu(&self) -> Vec<MenuItem<Self>> {
        vec![StandardItem {
            label: "Exit".into(),
            icon_name: "application-exit".into(),
            activate: Box::new(|_| std::process::exit(0)),
            ..Default::default()
        }
        .into()]
    }
}

impl KimeTray {
    pub fn new() -> Self {
        Self {
            icon_name: "kime-latin-black",
        }
    }
    pub fn update_with_bytes(&mut self, bytes: &[u8]) {
        if bytes.len() < 2 {
            return;
        }

        let category = match bytes[0] {
            1 => InputCategory::Hangul,
            _ => InputCategory::Latin,
        };

        let color = match bytes[1] {
            1 => IconColor::White,
            _ => IconColor::Black,
        };

        self.update(category, color);
    }

    pub fn update(&mut self, category: InputCategory, color: IconColor) {
        log::debug!("Update: ({:?}, {:?})", category, color);

        self.icon_name = match category {
            InputCategory::Latin => match color {
                IconColor::Black => "kime-latin-black",
                IconColor::White => "kime-latin-white",
            },
            InputCategory::Hangul => match color {
                IconColor::Black => "kime-hangul-black",
                IconColor::White => "kime-hangul-white",
            },
        }
    }
}

fn indicator_server(file_path: &Path) -> Result<()> {
    let service = ksni::TrayService::new(KimeTray::new());
    let handle = service.handle();
    service.spawn();

    std::fs::remove_file(file_path).ok();

    let listener = UnixListener::bind(file_path)?;

    let mut current_bytes = [0; 2];
    let mut read_buf = [0; 2];

    loop {
        let mut client = listener.accept()?.0;
        client.set_read_timeout(Some(Duration::from_secs(2))).ok();
        client.set_write_timeout(Some(Duration::from_secs(2))).ok();
        client.write_all(&current_bytes).ok();
        client.shutdown(Shutdown::Write).ok();
        match client.read_exact(&mut read_buf) {
            Ok(_) => {
                current_bytes = read_buf;

                handle.update(|tray| {
                    tray.update_with_bytes(&current_bytes);
                });
            }
            _ => {}
        }
    }
}

fn main() {
    kime_version::cli_boilerplate!((),);

    let run_dir = kime_run_dir::get_run_dir();
    let file_path = run_dir.join("kime-indicator.sock");
    indicator_server(&file_path).unwrap();
}
