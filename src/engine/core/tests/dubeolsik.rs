use kime_engine_core::{Config, InputEngine, InputResult, Key, KeyCode::*, RawConfig};

fn test_input_impl(word_commit: bool, keys: &[(Key, &str, &str)]) {
    let config = Config::from_raw_config(
        RawConfig {
            layout: "dubeolsik".into(),
            ..Default::default()
        },
        None,
    );

    let mut engine = InputEngine::new(word_commit);

    engine.set_hangul_enable(word_commit);

    for (key, preedit, commit) in keys.iter().copied() {
        eprintln!("Key: {:?}", key);

        let ret = engine.press_key(key, &config);

        dbg!(ret);

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

fn test_input(keys: &[(Key, &str, &str)]) {
    test_input_impl(false, keys)
}

fn test_word_input(keys: &[(Key, &str, &str)]) {
    test_input_impl(true, keys)
}

#[test]
fn word_hello() {
    test_word_input(&[
        (Key::normal(D), "ㅇ", ""),
        (Key::normal(K), "아", ""),
        (Key::normal(S), "안", ""),
        (Key::normal(S), "안ㄴ", ""),
        (Key::normal(U), "안녀", ""),
        (Key::normal(D), "안녕", ""),
        (Key::normal(Esc), "", "안녕PASS"),
    ])
}

#[test]
fn esc() {
    test_input(&[
        (Key::normal(R), "ㄱ", ""),
        (Key::normal(Esc), "", "ㄱPASS"),
        (Key::normal(R), "", "PASS"),
    ]);
}

#[test]
fn issue_28() {
    test_input(&[(Key::normal(K), "ㅏ", ""), (Key::normal(R), "가", "")])
}

#[test]
fn next_jaum() {
    test_input(&[
        (Key::normal(D), "ㅇ", ""),
        (Key::normal(K), "아", ""),
        (Key::normal(D), "앙", ""),
        (Key::normal(E), "ㄷ", "앙"),
    ])
}

#[test]
fn next_ssangjaum() {
    test_input(&[
        (Key::normal(A), "ㅁ", ""),
        (Key::normal(K), "마", ""),
        (Key::shift(T), "맜", ""),
        (Key::normal(K), "싸", "마"),
    ])
}

#[test]
fn not_com_moum_when_continue() {
    test_input(&[
        (Key::normal(D), "ㅇ", ""),
        (Key::normal(H), "오", ""),
        (Key::normal(D), "옹", ""),
        (Key::normal(K), "아", "오"),
    ]);
}

#[test]
fn com_moum() {
    test_input(&[
        (Key::normal(D), "ㅇ", ""),
        (Key::normal(H), "오", ""),
        (Key::normal(L), "외", ""),
        (Key::normal(D), "욍", ""),
        (Key::normal(D), "ㅇ", "욍"),
        (Key::normal(K), "아", ""),
        (Key::normal(S), "안", ""),
        (Key::normal(G), "않", ""),
        (Key::normal(E), "ㄷ", "않"),
    ]);
}

#[test]
fn number() {
    test_input(&[
        (Key::normal(D), "ㅇ", ""),
        (Key::normal(H), "오", ""),
        (Key::normal(L), "외", ""),
        (Key::normal(D), "욍", ""),
        (Key::normal(D), "ㅇ", "욍"),
        (Key::normal(K), "아", ""),
        (Key::normal(S), "안", ""),
        (Key::normal(G), "않", ""),
        (Key::normal(E), "ㄷ", "않"),
        (Key::normal(One), "", "ㄷ1"),
    ]);
}

#[test]
fn exclamation_mark() {
    test_input(&[(Key::shift(R), "ㄲ", ""), (Key::shift(One), "", "ㄲ!")]);
}

#[test]
fn backspace() {
    test_input(&[
        (Key::normal(R), "ㄱ", ""),
        (Key::normal(K), "가", ""),
        (Key::normal(D), "강", ""),
        (Key::normal(Backspace), "가", ""),
        (Key::normal(Q), "갑", ""),
        (Key::normal(T), "값", ""),
        (Key::normal(Backspace), "갑", ""),
        (Key::normal(Backspace), "가", ""),
        (Key::normal(Backspace), "ㄱ", ""),
        (Key::normal(Backspace), "", ""),
        (Key::normal(D), "ㅇ", ""),
        (Key::normal(H), "오", ""),
        (Key::normal(L), "외", ""),
        (Key::normal(Backspace), "오", ""),
        (Key::normal(Backspace), "ㅇ", ""),
        (Key::normal(Backspace), "", ""),
        (Key::normal(D), "ㅇ", ""),
        (Key::normal(H), "오", ""),
        (Key::normal(K), "와", ""),
        (Key::normal(Backspace), "오", ""),
        (Key::normal(Backspace), "ㅇ", ""),
        (Key::normal(Backspace), "", ""),
        (Key::normal(R), "ㄱ", ""),
    ])
}

#[test]
fn compose_jong() {
    test_input(&[
        (Key::normal(D), "ㅇ", ""),
        (Key::normal(J), "어", ""),
        (Key::normal(Q), "업", ""),
        (Key::normal(T), "없", ""),
    ])
}

#[test]
fn backspace_moum_compose() {
    test_input(&[
        (Key::normal(D), "ㅇ", ""),
        (Key::normal(H), "오", ""),
        (Key::normal(K), "와", ""),
        (Key::normal(Backspace), "오", ""),
        (Key::normal(Backspace), "ㅇ", ""),
    ])
}
