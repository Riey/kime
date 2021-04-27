#[doc(hidden)]
pub use kime_log;

#[doc(hidden)]
pub use kime_engine_cffi;

#[doc(hidden)]
pub mod build {
    pub const VERSION: &str = include_str!("../../../../VERSION");
}

#[macro_export]
macro_rules! cli_boilerplate {
    ($ok:expr, $($help:expr,)*) => {{
        let mut args = pico_args::Arguments::from_env();

        if args.contains(["-h", "--help"]) {
            println!("-h or --help: show help");
            println!("-v or --version: show version");
            println!("--log <level>: set logging level");
            $(
                println!($help);
            )*
            return $ok;
        }

        if args.contains(["-v", "--version"]) {
            $crate::print_version!();
            return $ok;
        }

        let log_level = args.opt_value_from_str("--log")
            .ok()
            .flatten()
            .unwrap_or_else(|| {
                $crate::kime_engine_cffi::LogConfig::load().global_level().parse().unwrap()
            });
        $crate::kime_log::enable_logger(log_level);

        args
    }};
}

#[macro_export]
macro_rules! print_version {
    () => {
        println!("kime {}", $crate::build::VERSION);
        println!("`{}` {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
    };
}
