use std::env;
use strum::{EnumIter, IntoStaticStr, IntoEnumIterator, EnumMessage, EnumProperty};
use ansi_term::Color;

#[derive(Clone, Copy, PartialEq, Eq, IntoStaticStr)]
enum CondResult {
    #[strum(to_string = "  (OK)  ")]
    Ok,
    #[strum(to_string = " (FAIL) ")]
    Fail,
    #[strum(to_string = "(IGNORE)")]
    Ignore,
}

impl CondResult {
    pub fn color(self) -> Color {
        match self {
            CondResult::Ok => Color::Green,
            CondResult::Fail => Color::Red,
            CondResult::Ignore => Color::Purple,
        }
    }
}

impl From<bool> for CondResult {
    fn from(b: bool) -> Self {
        match b {
            true => Self::Ok,
            false => Self::Fail,
        }
    }
}

#[derive(Clone, Copy, EnumIter, EnumMessage, EnumProperty)]
enum Check {
    #[strum(message = "Engine api version")]
    #[strum(props(fail = "Install correct kime engine"))]
    ApiVersion,
    #[strum(message = "XMODIFIERS env")]
    #[strum(props(fail = "Set XMODIFIERS=@im=kime", ignore = "Session is not x11"))]
    XModifier,
    #[strum(message = "GTK_IM_MODULE env")]
    #[strum(props(fail = "Set GTK_IM_MODUILE=kime"))]
    GtkImModule,
    #[strum(message = "QT_IM_MODULE env")]
    #[strum(props(fail = "Set QT_IM_MODULE=kime"))]
    QtImModule,
}

impl Check {
    pub fn cond(self) -> CondResult {
        match self {
            Check::ApiVersion => {
                kime_engine_cffi::check_api_version().into()
            }
            Check::XModifier => {
                match env::var("XDG_SESSION_TYPE").unwrap().as_str() {
                    "x11" => check_var("XMODIFIERS", "@im=kime"),
                    _ => CondResult::Ignore,
                }
            }
            Check::GtkImModule => {
                check_var("GTK_IM_MODULE", "kime")
            }
            Check::QtImModule => {
                check_var("QT_IM_MODULE", "kime")
            }
        }
    }
}

fn check_var(name: &str, value: &str) -> CondResult {
    env::var(name).map_or(false, |v| v.contains(value)).into()
}

fn main() {
    for check in Check::iter() {
        let ret = check.cond();
        let c = ret.color();

        print!("{} {:<30}", c.paint(<&str>::from(ret)), check.get_message().unwrap());

        match ret {
            CondResult::Ok => println!(),
            CondResult::Fail => {
                if let Some(fail) = check.get_str("fail") {
                    println!(" ({})", c.paint(fail));
                }
            }
            CondResult::Ignore => {
                if let Some(ignore) = check.get_str("ignore") {
                    println!(" ({})", c.paint(ignore));
                }
            }
        }
    }
}
