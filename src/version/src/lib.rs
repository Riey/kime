#[doc(hidden)]
pub mod build {
    include!(concat!(env!("OUT_DIR"), "/shadow.rs"));
}

#[macro_export]
macro_rules! print_version {
    () => {
        if $crate::build::TAG.is_empty() {
            println!("kime(git) {} {}", $crate::build::COMMIT_DATE, $crate::build::SHORT_COMMIT);
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
