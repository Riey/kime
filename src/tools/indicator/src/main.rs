use anyhow::Result;
use kime_engine_cffi::{IconColor, IndicatorConfig as Config};
use ksni::menu::*;
use std::net::Shutdown;
use std::os::unix::net::{UnixListener, UnixStream};
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

struct KimeTray {
    icon_name: &'static str,
    color: IconColor,
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

const fn icon_name(category: InputCategory, color: IconColor) -> &'static str {
    match (category, color) {
        (InputCategory::Latin, IconColor::Black) => "kime-latin-black",
        (InputCategory::Latin, IconColor::White) => "kime-latin-white",
        (InputCategory::Hangul, IconColor::Black) => "kime-hangul-black",
        (InputCategory::Hangul, IconColor::White) => "kime-hangul-white",
    }
}

impl KimeTray {
    pub fn new(color: IconColor) -> Self {
        Self {
            // Set init category Latin
            // TODO: should consider `default_category` config?
            icon_name: icon_name(InputCategory::Latin, color),
            color,
        }
    }
    pub fn update_with_bytes(&mut self, bytes: &[u8; 1]) {
        let category = match bytes[0] {
            1 => InputCategory::Hangul,
            _ => InputCategory::Latin,
        };

        self.update(category);
    }

    pub fn update(&mut self, category: InputCategory) {
        log::debug!("Update: {:?}", category);
        self.icon_name = icon_name(category, self.color);
    }
}

const EXIT_MESSAGE: &[u8; 1] = b"Z";

fn try_terminate_previous_server(file_path: &Path) -> Result<()> {
    let mut client = UnixStream::connect(file_path)?;

    client.write_all(EXIT_MESSAGE)?;

    Ok(())
}

fn indicator_server(file_path: &Path, color: IconColor) -> Result<()> {
    let service = ksni::TrayService::new(KimeTray::new(color));
    let handle = service.handle();
    service.spawn();

    if file_path.exists() {
        try_terminate_previous_server(file_path).ok();
        std::fs::remove_file(file_path).ok();
    }

    let listener = UnixListener::bind(file_path)?;

    let mut current_bytes = [0; 1];
    let mut read_buf = [0; 1];

    loop {
        let mut client = listener.accept()?.0;
        client.set_read_timeout(Some(Duration::from_secs(2))).ok();
        client.set_write_timeout(Some(Duration::from_secs(2))).ok();
        client.write_all(&current_bytes).ok();
        client.shutdown(Shutdown::Write).ok();
        match client.read_exact(&mut read_buf) {
            Ok(_) => {
                if &read_buf == EXIT_MESSAGE {
                    log::info!("Receive exit message");
                    return Ok(());
                }

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

    let config = Config::load();
    let run_dir = kime_run_dir::get_run_dir();
    let file_path = run_dir.join("kime-indicator.sock");
    indicator_server(&file_path, config.icon_color()).unwrap();
}
