#[doc(hidden)]
pub mod build {
    include!(concat!(env!("OUT_DIR"), "/shadow.rs"));
}

#[macro_export]
macro_rules! cli_boilerplate {
    () => {
        let mut args = pico_args::Arguments::from_env();

        if args.contains(["-h", "--help"]) {
            println!("-h or --help: show help");
            println!("-v or --version: show version");
            println!("--verbose: show verbose log");
            return;
        }

        if args.contains(["-v", "--version"]) {
            $crate::print_version!();
            return;
        }

        let level = if args.contains("--verbose") {
            kime_log::LevelFilter::Trace
        } else {
            kime_log::LevelFilter::Info
        };

        kime_log::enable_logger(level);

        log::info!(
            "Start {}: {}",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION")
        );
    };
}

#[macro_export]
macro_rules! print_version {
    () => {
        if $crate::build::TAG.is_empty() {
            println!(
                "kime(git) {} {}",
                $crate::build::COMMIT_DATE,
                $crate::build::SHORT_COMMIT
            );
        } else {
            println!("kime(release) {}", $crate::build::TAG);
        }
        println!("`{}` {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
    };
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
