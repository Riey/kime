use simplelog::*;

pub use simplelog::LevelFilter;

fn config() -> Config {
    ConfigBuilder::new()
        .set_level_padding(LevelPadding::Left)
        .set_time_level(LevelFilter::Trace)
        .build()
}

pub fn enable_logger(level: LevelFilter) -> bool {
    TermLogger::init(level, config(), TerminalMode::Stderr, ColorChoice::Auto).is_ok()
}
