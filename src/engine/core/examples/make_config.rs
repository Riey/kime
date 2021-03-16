fn main() {
    print!(
        "{}",
        serde_yaml::to_string(&kime_engine::RawConfig::default()).unwrap()
    );
}
