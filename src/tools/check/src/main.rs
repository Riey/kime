use kime_engine_core::{Key, KeyMap};

use ansi_term::Color;
use kime_engine_cffi::{
    Config, InputCategory, InputEngine, InputResult_CONSUMED, InputResult_HAS_COMMIT,
    InputResult_HAS_PREEDIT,
};
use pad::PadStr;
use std::env;
use std::io::BufRead;
use strum::{EnumIter, EnumMessage, IntoEnumIterator, IntoStaticStr};

#[derive(Clone, PartialEq, Eq, IntoStaticStr)]
enum CondResult {
    Ok,
    Fail(String),
    Ignore(String),
}

impl CondResult {
    pub const fn success(&self) -> bool {
        matches!(self, Self::Ok | Self::Ignore(..))
    }

    pub fn color(&self) -> Color {
        match self {
            CondResult::Ok => Color::Green,
            CondResult::Fail(..) => Color::Red,
            CondResult::Ignore(..) => Color::Purple,
        }
    }

    pub fn print(&self, message: &str) {
        let c = self.color();
        print!(
            "{} {}",
            c.paint(<&str>::from(self).pad_to_width(8)),
            Color::White.bold().paint(message.pad_to_width(40))
        );

        match self {
            CondResult::Ok => println!(),
            CondResult::Fail(msg) | CondResult::Ignore(msg) => println!("({})", c.paint(msg)),
        }
    }
}

#[derive(Clone, Copy, EnumIter, EnumMessage)]
enum Check {
    #[strum(message = "Engine api version")]
    ApiVersion,
    #[strum(message = "Check icons exists")]
    Icons,
    #[strum(message = "Config file")]
    Config,
    #[strum(message = "Engine works")]
    EngineWorks,
    #[strum(message = "XMODIFIERS has @im=kime")]
    XModifier,
    #[strum(message = "GTK_IM_MODULE has kime")]
    GtkImModule,
    #[strum(message = "QT_IM_MODULE has kime")]
    QtImModule,
    #[strum(message = "LANG has UTF-8")]
    Lang,
    #[strum(message = "Plasma virtual keyboard has kime")]
    PlasmaVirtualKeyboard,
}

impl Check {
    pub fn cond(self) -> CondResult {
        match self {
            Check::ApiVersion => {
                println!("KIME_API_VERSION: {}", kime_engine_cffi::KIME_API_VERSION);
                if kime_engine_cffi::check_api_version() {
                    CondResult::Ok
                } else {
                    CondResult::Fail("Install correct kime engine".into())
                }
            }
            Check::Icons => {
                let dirs = xdg::BaseDirectories::new().expect("Load xdg dirs");

                let icons = &[
                    "kime-hangul-black.png",
                    "kime-hangul-white.png",
                    "kime-latin-black.png",
                    "kime-latin-white.png",
                ];

                for icon in icons {
                    match dirs.find_data_file(format!("icons/hicolor/64x64/apps/{}", icon)) {
                        Some(path) => println!("Found icon: {}", path.display()),
                        _ => return CondResult::Fail(format!("Can't find icon {}", icon)),
                    }
                }

                CondResult::Ok
            }
            Check::EngineWorks => {
                let config = kime_engine_cffi::Config::default();
                let mut engine = kime_engine_cffi::InputEngine::new(&config);

                check_input(
                    &mut engine,
                    &config,
                    &[
                        // R
                        (27, "ㄱ", ""),
                        // K
                        (45, "가", ""),
                        // S
                        (39, "간", ""),
                        // K
                        (45, "나", "가"),
                    ],
                )
            }
            Check::Config => {
                let dirs = xdg::BaseDirectories::with_prefix("kime").expect("Load xdg dirs");
                let config_path = match dirs.find_config_file("config.yaml") {
                    Some(path) => path,
                    _ => {
                        return CondResult::Ignore(
                            "User config does not exists will use default config".into(),
                        )
                    }
                };

                println!("Loading config path: {}", config_path.display());

                let config: kime_engine_core::RawConfig = match serde_yaml::from_str(
                    &std::fs::read_to_string(config_path).expect("Read config file"),
                ) {
                    Ok(config) => config,
                    Err(err) => return CondResult::Fail(format!("Can't parse config.yaml: {err}")),
                };

                match config.engine.translation_layer {
                    Some(ref raw_path) => {
                        let path = match dirs.find_config_file(raw_path) {
                            Some(path) => path,
                            _ => {
                                return CondResult::Ignore(
                                    "translation layer configuration does not exist. No translation layer will be used".into())
                            }
                        };
                        println!("Loading translation layer config: {}", path.display());

                        let _translation_layer: KeyMap<Key> = match serde_yaml::from_str(
                            &std::fs::read_to_string(path.as_path())
                                .expect("Read translation layer config"),
                        ) {
                            Ok(c) => c,
                            Err(err) => {
                                return CondResult::Fail(format!("Can't parse {path:#?}: {err}"))
                            }
                        };
                    }
                    None => return CondResult::Ok,
                };

                // TODO: check layout

                CondResult::Ok
            }
            Check::XModifier => match env::var("XDG_SESSION_TYPE").unwrap().as_str() {
                "x11" => check_var(
                    "XMODIFIERS",
                    |v| v.contains("@im=kime"),
                    "set XMODIFIERS=@im=kime",
                ),
                other => CondResult::Ignore(format!("Session type is {} not x11", other)),
            },
            Check::GtkImModule => {
                check_var("GTK_IM_MODULE", |v| v == "kime", "set GTK_IM_MODULE=kime")
            }
            Check::QtImModule => {
                check_var("QT_IM_MODULE", |v| v == "kime", "set QT_IM_MODULE=kime")
            }
            Check::Lang => check_var(
                "LANG",
                |v| {
                    let v = v.to_ascii_lowercase();
                    v.ends_with("utf-8") || v.ends_with("utf8")
                },
                "set LANG encoding UTF-8",
            ),
            Check::PlasmaVirtualKeyboard => {
                let current_desktop = env::var("XDG_CURRENT_DESKTOP").map_or(String::new(), |x| x);
                let session_type = env::var("XDG_SESSION_TYPE").map_or(String::new(), |x| x);
                if current_desktop.contains("KDE") && session_type == "wayland" {
                    let dirs = xdg::BaseDirectories::new().expect("Load xdg dirs");
                    let config_path = match dirs.find_config_file("kwinrc") {
                        Some(path) => path,
                        _ => {
                            return CondResult::Fail(
                                "kwinrc configuration file doesn't exist".into(),
                            )
                        }
                    };

                    println!("Loading kwinrc: {}", config_path.display());

                    let file = std::fs::File::open(config_path).expect("Open kwinrc");
                    let lines = std::io::BufReader::new(file).lines();

                    let mut given_input_method = String::new();

                    for line in lines {
                        if let Ok(s) = line {
                            if s.contains("=") {
                                let splits: Vec<&str> = s.split('=').collect();
                                if splits[0].contains("InputMethod") {
                                    if splits[1].contains("kime.desktop") {
                                        return CondResult::Ok;
                                    } else {
                                        given_input_method = String::from(splits[1]);
                                    }
                                }
                            }
                        }
                    }

                    if given_input_method.is_empty() {
                        CondResult::Fail("Virtual keyboard is not set".to_string())
                    } else {
                        CondResult::Fail(format!(
                            "Virtual keyboard is set to {} not kime",
                            given_input_method
                        ))
                    }
                } else {
                    CondResult::Ignore(format!(
                        "Current desktop and session type is {} and {}, not KDE and wayland",
                        current_desktop, session_type
                    ))
                }
            }
        }
    }
}

fn check_input(
    engine: &mut InputEngine,
    config: &Config,
    tests: &[(u16, &str, &str)],
) -> CondResult {
    engine.set_input_category(InputCategory::Hangul);

    for (key, preedit, commit) in tests.iter().copied() {
        let ret = engine.press_key(config, key, false, 0);

        let preedit_ret;
        let commit_ret;

        if ret & InputResult_HAS_PREEDIT != 0 {
            preedit_ret = preedit == engine.preedit_str();
        } else {
            preedit_ret = preedit.is_empty();
        }

        if ret & InputResult_CONSUMED == 0 {
            commit_ret = commit == format!("{}PASS", engine.commit_str());
        } else if ret & InputResult_HAS_COMMIT != 0 {
            commit_ret = commit == engine.commit_str();
            engine.clear_commit();
        } else {
            commit_ret = commit.is_empty();
        }

        if !preedit_ret {
            return CondResult::Fail("Preedit result failed".into());
        }

        if !commit_ret {
            return CondResult::Fail("Commit result failed".into());
        }
    }

    CondResult::Ok
}

fn check_var(name: &str, pred: impl Fn(&str) -> bool, reason: &str) -> CondResult {
    if env::var(name).map_or(false, |v| pred(&v)) {
        CondResult::Ok
    } else {
        CondResult::Fail(reason.into())
    }
}

#[derive(Debug)]
struct CheckFailedError;

fn main() -> Result<(), CheckFailedError> {
    let mut success = true;

    for check in Check::iter() {
        let ret = check.cond();

        success = success && ret.success();

        ret.print(check.get_message().unwrap());
    }

    if success {
        Ok(())
    } else {
        Err(CheckFailedError)
    }
}
