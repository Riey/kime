use std::{env, path::PathBuf};

fn main() {
    println!("cargo:rustc-link-lib=dylib=kime_engine");

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let binding_path = out_dir.join("bindings.rs");
    let header_path = "kime_engine.h";

    let cbindings = cbindgen::generate("../capi").expect("Unable to generate C bindings");
    cbindings.write_to_file(header_path);

    let bindings = bindgen::Builder::default()
        .header("./kime_engine.h")
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
