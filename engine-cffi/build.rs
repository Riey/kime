use std::{env, path::PathBuf};

fn main() {
    println!("cargo:rustc-link-lib=kime_engine");

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let header_path = "kime_engine.h";
    let binding_path = out_dir.join("bindings.rs");

    let cbindings = cbindgen::Builder::new()
        .with_no_includes()
        .with_sys_include("stdint.h")
        .with_include_version(true)
        .with_pragma_once(true)
        .with_autogen_warning("/* DO NOT MODIFY THIS MANUALLY */")
        .with_braces(cbindgen::Braces::SameLine)
        .with_language(cbindgen::Language::C)
        .with_style(cbindgen::Style::Both)
        .with_crate("../engine")
        .with_item_prefix("Kime")
        .generate()
        .expect("Unable to generate C bindings");

    cbindings.write_to_file(header_path);

    let bindings = bindgen::Builder::default()
        .header(header_path)
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
