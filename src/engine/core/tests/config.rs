#[test]
fn check_default_config() {
    assert_eq!(
        serde_yaml::to_string(&kime_engine_core::RawConfig::default()).unwrap(),
        include_str!("../../../../res/default_config.yaml")
    );
}
