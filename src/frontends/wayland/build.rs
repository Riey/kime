use std::env;
use std::path::PathBuf;
use wayland_scanner::{generate_code, Side};

fn main() {
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());

    generate_code(
        "./protocol/input-method-unstable-v2.xml",
        out.join("input_method_api.rs"),
        Side::Client,
    );

    println!("cargo:rerun-if-changed=Cargo.toml");
    println!("cargo:rerun-if-changed=./protocols/");
}
