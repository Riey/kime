fn main() {
    std::fs::write(
        "data/config.yaml",
        serde_yaml::to_string(&kime_engine_core::RawConfig::default()).unwrap(),
    )
    .unwrap();
}
