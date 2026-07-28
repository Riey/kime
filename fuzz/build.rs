fn main() {
    // Only the engine_diff_libhangul target calls into libhangul, but
    // cargo links the lib crate into every bin; probe once here.
    pkg_config::Config::new()
        .probe("libhangul")
        .expect("libhangul not found — install it or set PKG_CONFIG_PATH (see fuzz/README.md)");
}
