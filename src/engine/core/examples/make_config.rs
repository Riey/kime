fn main() {
    print!(
        "{}",
        serde_yaml::to_string(&kime_engine_core::RawConfig::default()).unwrap()
    );
}
