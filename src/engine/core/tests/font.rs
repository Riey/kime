use kime_engine_core::load_engine_config_from_config_dir;

#[test]
fn font() {
    let config = load_engine_config_from_config_dir().unwrap();
    assert!(!config.candidate_font.0.is_empty());
}
