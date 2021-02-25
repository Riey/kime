use ansi_term::Color;
use kime_engine_cffi::{
    Config, InputEngine, InputResult_CONSUMED, InputResult_HAS_PREEDIT, InputResult_NEED_FLUSH,
    InputResult_NEED_RESET,
};
use pad::PadStr;
use std::env;
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
            Color::White.bold().paint(message.pad_to_width(30))
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
}

impl Check {
    pub fn cond(self) -> CondResult {
        match self {
            Check::ApiVersion => {
                if kime_engine_cffi::check_api_version() {
                    CondResult::Ok
                } else {
                    CondResult::Fail("Install correct kime engine".into())
                }
            }
            Check::Icons => {
                let dirs = xdg::BaseDirectories::with_prefix("kime").expect("Load xdg dirs");

                let icons = &[
                    "kime-han-black-64x64.png",
                    "kime-han-white-64x64.png",
                    "kime-eng-black-64x64.png",
                    "kime-eng-white-64x64.png",
                ];

                for icon in icons {
                    match dirs.find_data_file(format!("icons/{}", icon)) {
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
                    _ => return CondResult::Fail("Can't find config.yaml".into()),
                };

                println!("Loading config path: {}", config_path.display());

                let _config: kime_engine_core::RawConfig = match serde_yaml::from_str(
                    &std::fs::read_to_string(config_path).expect("Read config file"),
                ) {
                    Ok(config) => config,
                    Err(err) => {
                        return CondResult::Fail(format!("Can't parse config.yaml: {}", err))
                    }
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
                |v| v.to_ascii_lowercase().ends_with("utf-8"),
                "set LANG encoding UTF-8",
            ),
        }
    }
}

fn check_input(
    engine: &mut InputEngine,
    config: &Config,
    tests: &[(u16, &str, &str)],
) -> CondResult {
    engine.set_hangul_enable(true);

    for (key, preedit, commit) in tests.iter().copied() {
        let ret = engine.press_key(config, key, 0);

        let preedit_ret;
        let commit_ret;

        if ret & InputResult_HAS_PREEDIT != 0 {
            preedit_ret = preedit == engine.preedit_str();
        } else {
            preedit_ret = preedit.is_empty();
        }

        if ret & InputResult_CONSUMED == 0 {
            commit_ret = commit == format!("{}PASS", engine.commit_str());
        } else if ret & (InputResult_NEED_RESET | InputResult_NEED_FLUSH) != 0 {
            commit_ret = commit == engine.commit_str();
        } else {
            commit_ret = commit.is_empty();
        }

        if !preedit_ret {
            return CondResult::Fail("Preedit result failed".into());
        }

        if !commit_ret {
            return CondResult::Fail("Commit result failed".into());
        }

        if ret & InputResult_NEED_FLUSH != 0 {
            engine.flush();
        }

        if ret & InputResult_NEED_RESET != 0 {
            engine.reset();
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
