use simplelog::*;

#[derive(Clone, Copy)]
#[repr(u32)]
pub enum LogLevel {
    Off,
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

fn config() -> Config {
    ConfigBuilder::new()
        .set_level_color(Level::Error, Color::Red)
        .set_level_color(Level::Warn, Color::Yellow)
        .set_level_color(Level::Info, Color::Green)
        .set_level_color(Level::Debug, Color::Blue)
        .set_level_color(Level::Trace, Color::Black)
        .set_level_padding(LevelPadding::Left)
        .set_time_level(LevelFilter::Trace)
        .build()
}

pub fn enable_logger(level: LogLevel) -> bool {
    SimpleLogger::init(
        match level {
            LogLevel::Trace => LevelFilter::Trace,
            LogLevel::Debug => LevelFilter::Debug,
            LogLevel::Info => LevelFilter::Info,
            LogLevel::Warn => LevelFilter::Warn,
            LogLevel::Error => LevelFilter::Error,
            LogLevel::Off => LevelFilter::Off,
        },
        config(),
    )
    .is_ok()
}

pub fn enable_logger_with_env() -> bool {
    match std::env::var("KIME_LOG") {
        Ok(mut v) => {
            v.make_ascii_uppercase();

            let level = match v.as_str() {
                "TRACE" => LogLevel::Trace,
                "DEBUG" => LogLevel::Debug,
                "INFO" => LogLevel::Info,
                "WARN" => LogLevel::Warn,
                "ERROR" => LogLevel::Error,
                _ => LogLevel::Off,
            };

            enable_logger(level)
        }
        _ => enable_logger(LogLevel::Off),
    }
}
