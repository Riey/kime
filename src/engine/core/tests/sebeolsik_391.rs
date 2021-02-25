use kime_engine_core::{Config, InputEngine, InputResult, Key, KeyCode::*, RawConfig};

#[track_caller]
fn test_input(keys: &[(Key, &str, &str)]) {
    let config = Config::from_raw_config(
        RawConfig {
            layout: "sebeolsik-391".into(),
            ..Default::default()
        },
        None,
    );

    let mut engine = InputEngine::new(false);

    engine.set_hangul_enable(true);

    for (key, preedit, commit) in keys.iter().copied() {
        eprintln!("Key: {:?}", key);

        let ret = engine.press_key(key, &config);

        eprintln!("Ret: {:?}", ret);

        if ret.contains(InputResult::HAS_PREEDIT) {
            assert_eq!(preedit, engine.preedit_str());
        } else {
            assert!(preedit.is_empty());
        }

        if !ret.contains(InputResult::CONSUMED) {
            assert_eq!(commit, format!("{}PASS", engine.commit_str()));
        } else if ret.intersects(InputResult::NEED_RESET | InputResult::NEED_FLUSH) {
            assert_eq!(commit, engine.commit_str());
        } else {
            assert!(commit.is_empty());
        }

        if ret.contains(InputResult::NEED_RESET) {
            engine.reset();
        } else if ret.contains(InputResult::NEED_FLUSH) {
            engine.flush();
        }
    }
}

#[test]
fn hello() {
    test_input(&[
        (Key::normal(J), "ㅇ", ""),
        (Key::normal(F), "아", ""),
        (Key::normal(S), "안", ""),
        (Key::normal(H), "ㄴ", "안"),
        (Key::normal(E), "녀", ""),
        (Key::normal(A), "녕", ""),
    ]);
}

#[test]
fn switch_next() {
    test_input(&[
        (Key::normal(S), "ㄴ", ""),
        (Key::normal(J), "ㅇ", "ㄴ"),
        (Key::normal(F), "아", ""),
    ]);
}

#[test]
fn good() {
    test_input(&[
        (Key::normal(K), "ㄱ", ""),
        (Key::normal(R), "개", ""),
        (Key::normal(K), "ㄱ", "개"),
        (Key::normal(K), "ㄲ", ""),
        (Key::normal(B), "꾸", ""),
        (Key::normal(W), "꿀", ""),
    ])
}

#[test]
fn colon() {
    test_input(&[(Key::normal(Backslash), "", ":")]);
}
