fn main() {
    println!(
        "{}",
        serde_yaml::to_string(&kime_engine_core::RawConfig::default()).unwrap()
    );
}
