use std::env;

fn main() {
    println!("cargo:rerun-if-changed=./kime_engine.h");
    println!("cargo:rerun-if-changed=./kime_engine.hpp");
    println!("cargo:rerun-if-changed=../capi");

    let c_binding = cbindgen::generate_with_config(
        "../capi",
        cbindgen::Config::from_file("../capi/cbindgen-c.toml").unwrap(),
    )
    .unwrap();

    c_binding.write_to_file("kime_engine.h");

    let cpp_binding = cbindgen::generate_with_config(
        "../capi",
        cbindgen::Config::from_file("../capi/cbindgen-cpp.toml").unwrap(),
    )
    .unwrap();

    cpp_binding.write_to_file("kime_engine.hpp");

    let rust_binding = bindgen::builder()
        .layout_tests(false)
        .header("./kime_engine.hpp")
        .disable_name_namespacing()
        .rustified_enum("kime::.+")
        .allowlist_var("kime::.+")
        .allowlist_type("kime::.+")
        .allowlist_function("kime::.+")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .unwrap();

    rust_binding
        .write_to_file(std::path::PathBuf::from(env::var("OUT_DIR").unwrap()).join("bindings.rs"))
        .unwrap();
}
