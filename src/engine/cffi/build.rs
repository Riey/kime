use std::{env, path::PathBuf, process::Command};

fn main() {
    println!("cargo:rustc-link-lib=dylib=kime_engine");

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let header_path = "kime_engine.h";
    let binding_path = out_dir.join("bindings.rs");

    assert!(Command::new("cbindgen")
        .arg("../capi")
        .arg("--output")
        .arg(header_path)
        .spawn()
        .unwrap()
        .wait()
        .unwrap()
        .success());

    let bindings = bindgen::Builder::default()
        .header(header_path)
        .whitelist_var("Kime.*")
        .whitelist_function("kime_.*")
        .whitelist_type("Kime_.*")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .default_enum_style(bindgen::EnumVariation::Rust {
            non_exhaustive: false,
        })
        .generate()
        .expect("Unable to generate bindings");

    bindings.write_to_file(binding_path).unwrap();
}
