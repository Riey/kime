use std::{env, path::PathBuf};

fn main() {
    println!("cargo:rustc-link-lib=kime-engine");
    println!("cargo:rerun-if-changed=../engine");

    let bindings = bindgen::Builder::default()
        .header("../engine/kime-engine.h")
        .whitelist_function("kime_.*")
        .whitelist_type("Kime_.*")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .default_enum_style(bindgen::EnumVariation::Rust {
            non_exhaustive: false,
        })
        .generate()
        .expect("Unable to generate bindings");

    bindings
        .write_to_file(PathBuf::from(env::var("OUT_DIR").unwrap()).join("bindings.rs"))
        .unwrap();
}
