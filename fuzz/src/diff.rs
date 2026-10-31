//! Differential harness: the same 2-set (dubeolsik) key sequence through
//! kime's `InputEngine` and libhangul's `HangulInputContext` (the
//! composition engine behind ibus-hangul and fcitx5-hangul) must yield the
//! same text.
//!
//! Comparison model: after every op, `committed-so-far + current preedit`
//! must be identical on both sides. Confirmed-benign design differences get
//! absorbed in [`normalize`] — grow it only from triaged findings, never
//! speculatively.

use arbitrary::Arbitrary;
use kime_engine_backend::{Key, KeyCode, ModifierState};
use kime_engine_config::{EngineConfig, InputCategory};
use kime_engine_core::{Config, InputEngine, InputResult};
use std::ffi::c_char;
use std::sync::LazyLock;

#[allow(non_camel_case_types)]
type ucschar = u32;

#[repr(C)]
struct HangulInputContext {
    _priv: [u8; 0],
}

extern "C" {
    fn hangul_ic_new(keyboard: *const c_char) -> *mut HangulInputContext;
    fn hangul_ic_delete(hic: *mut HangulInputContext);
    fn hangul_ic_process(hic: *mut HangulInputContext, ascii: i32) -> bool;
    fn hangul_ic_backspace(hic: *mut HangulInputContext) -> bool;
    fn hangul_ic_flush(hic: *mut HangulInputContext) -> *const ucschar;
    fn hangul_ic_get_preedit_string(hic: *mut HangulInputContext) -> *const ucschar;
    fn hangul_ic_get_commit_string(hic: *mut HangulInputContext) -> *const ucschar;
}

struct Libhangul(*mut HangulInputContext);

impl Libhangul {
    fn new() -> Self {
        let hic = unsafe { hangul_ic_new(c"2".as_ptr()) };
        assert!(!hic.is_null(), "hangul_ic_new failed");
        Self(hic)
    }
    fn process(&mut self, ascii: char) -> bool {
        unsafe { hangul_ic_process(self.0, ascii as i32) }
    }
    fn backspace(&mut self) -> bool {
        unsafe { hangul_ic_backspace(self.0) }
    }
    fn flush(&mut self) -> String {
        unsafe { ucs4_str(hangul_ic_flush(self.0)) }
    }
    fn preedit(&self) -> String {
        unsafe { ucs4_str(hangul_ic_get_preedit_string(self.0)) }
    }
    fn commit(&self) -> String {
        unsafe { ucs4_str(hangul_ic_get_commit_string(self.0)) }
    }
}

impl Drop for Libhangul {
    fn drop(&mut self) {
        unsafe { hangul_ic_delete(self.0) }
    }
}

unsafe fn ucs4_str(mut p: *const ucschar) -> String {
    let mut s = String::new();
    while !p.is_null() && *p != 0 {
        s.push(char::from_u32(*p).unwrap_or('\u{FFFD}'));
        p = p.add(1);
    }
    s
}

/// A dubeolsik key: the kime keycode plus the ASCII byte libhangul reads.
/// Shifted keys carry the capital letter, which is how libhangul is told
/// about the shift state.
type DubeolKey = (KeyCode, char, bool);

macro_rules! keys {
    ($(($code:ident, $ascii:literal, $shift:literal)),* $(,)?) => {
        &[$((KeyCode::$code, $ascii, $shift)),*]
    };
}

/// Onset consonants, plain and doubled.
const ONSETS: &[DubeolKey] = keys![
    (R, 'r', false), // ㄱ
    (R, 'R', true),  // ㄲ
    (S, 's', false), // ㄴ
    (E, 'e', false), // ㄷ
    (E, 'E', true),  // ㄸ
    (F, 'f', false), // ㄹ
    (A, 'a', false), // ㅁ
    (Q, 'q', false), // ㅂ
    (Q, 'Q', true),  // ㅃ
    (T, 't', false), // ㅅ
    (T, 'T', true),  // ㅆ
    (D, 'd', false), // ㅇ
    (W, 'w', false), // ㅈ
    (W, 'W', true),  // ㅉ
    (C, 'c', false), // ㅊ
    (Z, 'z', false), // ㅋ
    (X, 'x', false), // ㅌ
    (V, 'v', false), // ㅍ
    (G, 'g', false), // ㅎ
];

/// Vowels. Compound vowels (ㅘㅙㅚㅝㅞㅟㅢ) are typed as two of these and
/// arise on their own from consecutive `vowel` picks inside a syllable.
const VOWELS: &[DubeolKey] = keys![
    (K, 'k', false), // ㅏ
    (O, 'o', false), // ㅐ
    (I, 'i', false), // ㅑ
    (O, 'O', true),  // ㅒ
    (J, 'j', false), // ㅓ
    (P, 'p', false), // ㅔ
    (U, 'u', false), // ㅕ
    (P, 'P', true),  // ㅖ
    (H, 'h', false), // ㅗ
    (Y, 'y', false), // ㅛ
    (N, 'n', false), // ㅜ
    (B, 'b', false), // ㅠ
    (M, 'm', false), // ㅡ
    (L, 'l', false), // ㅣ
];

/// Consonants that can stand as a final. ㄸㅃㅉ are excluded: they are not
/// valid jongseong, so they would re-open the consonant-run divergence
/// this structure exists to avoid.
const CODAS: &[DubeolKey] = keys![
    (R, 'r', false), // ㄱ
    (R, 'R', true),  // ㄲ
    (S, 's', false), // ㄴ
    (E, 'e', false), // ㄷ
    (F, 'f', false), // ㄹ
    (A, 'a', false), // ㅁ
    (Q, 'q', false), // ㅂ
    (T, 't', false), // ㅅ
    (T, 'T', true),  // ㅆ
    (D, 'd', false), // ㅇ
    (W, 'w', false), // ㅈ
    (C, 'c', false), // ㅊ
    (Z, 'z', false), // ㅋ
    (X, 'x', false), // ㅌ
    (V, 'v', false), // ㅍ
    (G, 'g', false), // ㅎ
];

/// Input is generated as whole syllables rather than free key sequences.
///
/// kime and libhangul genuinely disagree on consonant runs — kime commits
/// a lone jamo where libhangul keeps building a cluster in its preedit, so
/// `ㅂ ㅅ ㅖ` ends as `ㅂㅖ` in one and `볘` in the other. That difference
/// is real but it is not what this target is for, and it is so easy to
/// reach that a free-form generator never gets past it. Well-formed
/// syllables keep both engines in the region where they agree, which is
/// where a dropped or duplicated final syllable actually shows up.
#[derive(Arbitrary, Debug)]
pub enum DiffOp {
    Syllable {
        onset: u8,
        vowel: u8,
        /// A second vowel key, producing a compound vowel when the pair
        /// composes (ㅗ+ㅏ) — the shape most last-syllable bugs need.
        vowel2: Option<u8>,
        coda: Option<u8>,
    },
    Backspace,
    Flush,
}

/// Strict dubeolsik: ssang-compose addon off (libhangul doesn't do ㄱ+ㄱ=ㄲ),
/// jongseong-resplit on (inherent to libhangul's 2-set automaton).
static DIFF_CONFIG: LazyLock<Config> = LazyLock::new(|| {
    let mut engine = EngineConfig::default();
    engine.default_category = InputCategory::Hangul;
    engine.global_category_state = false;
    engine.global_hotkeys.clear();
    engine.category_hotkeys.clear();
    engine.mode_hotkeys.clear();
    engine.hangul.layout = "dubeolsik".into();
    engine.hangul.word_commit = false;
    engine.hangul.addons = [(
        "dubeolsik".to_string(),
        kime_engine_backend_hangul::Addon::TreatJongseongAsChoseong.into(),
    )]
    .into_iter()
    .collect();
    Config::new(engine)
});

/// Compatibility jamo that stand for two letters, and the pair each one
/// spells out. Ssang jamo (ㄲㄸㅃㅆㅉ) are deliberately absent: those are
/// reachable by a single shifted key, so folding them would hide a real
/// difference between "typed Shift-ㄱ" and "typed ㄱ twice".
const CLUSTER_JAMO: &[(char, &str)] = &[
    ('ㄳ', "ㄱㅅ"),
    ('ㄵ', "ㄴㅈ"),
    ('ㄶ', "ㄴㅎ"),
    ('ㄺ', "ㄹㄱ"),
    ('ㄻ', "ㄹㅁ"),
    ('ㄼ', "ㄹㅂ"),
    ('ㄽ', "ㄹㅅ"),
    ('ㄾ', "ㄹㅌ"),
    ('ㄿ', "ㄹㅍ"),
    ('ㅀ', "ㄹㅎ"),
    ('ㅄ', "ㅂㅅ"),
    ('ㅘ', "ㅗㅏ"),
    ('ㅙ', "ㅗㅐ"),
    ('ㅚ', "ㅗㅣ"),
    ('ㅝ', "ㅜㅓ"),
    ('ㅞ', "ㅜㅔ"),
    ('ㅟ', "ㅜㅣ"),
    ('ㅢ', "ㅡㅣ"),
];

/// Absorb confirmed-benign representation differences.
///
/// So far exactly one: a lone jamo pair with no vowel to attach to.
/// Typing `ㄹ` then `ㅂ` leaves kime showing `ㄹㅂ` and libhangul showing
/// the cluster jamo `ㄼ` — same two letters, same order, one glyph versus
/// two. Spelling clusters out makes the two comparable without touching
/// composed syllables (가-힣), so a dropped or duplicated final syllable
/// still diverges loudly.
fn normalize(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match CLUSTER_JAMO.iter().find(|(c, _)| *c == ch) {
            Some((_, pair)) => out.push_str(pair),
            None => out.push(ch),
        }
    }
    out
}

const MAX_OPS: usize = 128;

pub fn run_diff(ops: &[DiffOp]) {
    let config = &*DIFF_CONFIG;
    let mut kime = InputEngine::new(config);
    let mut lh = Libhangul::new();
    let mut kime_acc = String::new();
    let mut lh_acc = String::new();

    for (i, op) in ops.iter().take(MAX_OPS).enumerate() {
        match op {
            DiffOp::Syllable {
                onset,
                vowel,
                vowel2,
                coda,
            } => {
                let mut press = |table: &[DubeolKey], idx: u8| {
                    let (code, ascii, shift) = table[idx as usize % table.len()];
                    let state = if shift {
                        ModifierState::SHIFT
                    } else {
                        ModifierState::empty()
                    };
                    let ret = kime.press_key(Key::new(code, state), config);
                    if ret.contains(InputResult::HAS_COMMIT) {
                        kime_acc.push_str(kime.commit_str());
                        kime.clear_commit();
                    }
                    lh.process(ascii);
                    lh_acc.push_str(&lh.commit());
                };

                press(ONSETS, *onset);
                press(VOWELS, *vowel);
                if let Some(v2) = vowel2 {
                    press(VOWELS, *v2);
                }
                if let Some(c) = coda {
                    press(CODAS, *c);
                }
            }
            DiffOp::Backspace => {
                let ret =
                    kime.press_key(Key::new(KeyCode::Backspace, ModifierState::empty()), config);
                if ret.contains(InputResult::HAS_COMMIT) {
                    kime_acc.push_str(kime.commit_str());
                    kime.clear_commit();
                }
                // A backspace the engine doesn't take reaches the
                // application, which deletes the last character it already
                // holds. Both engines must be modelled this way or the
                // comparison just measures who commits earlier: kime
                // commits a lone jamo that libhangul is still holding in
                // its preedit, so only one of them sees the key.
                if !ret.contains(InputResult::CONSUMED) {
                    kime_acc.pop();
                }
                if !lh.backspace() {
                    lh_acc.pop();
                }
            }
            DiffOp::Flush => {
                // moves preedit into commit_buf
                kime.clear_preedit();
                kime_acc.push_str(kime.commit_str());
                kime.clear_commit();
                lh_acc.push_str(&lh.flush());
            }
        }

        let kime_text = normalize(&format!("{kime_acc}{}", kime.preedit_str()));
        let lh_text = normalize(&format!("{lh_acc}{}", lh.preedit()));
        assert_eq!(
            kime_text,
            lh_text,
            "divergence after op {i} ({op:?}) of {:?}",
            &ops[..=i]
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn idx(table: &[DubeolKey], ascii: char) -> u8 {
        table
            .iter()
            .position(|(_, a, _)| *a == ascii)
            .unwrap_or_else(|| panic!("{ascii} is not in the table")) as u8
    }

    /// A syllable written the way it is typed, e.g. `syl("g", "k", "s")`
    /// for 한 (ㅎㅏㄴ).
    fn syl(onset: char, vowel: char, coda: Option<char>) -> DiffOp {
        DiffOp::Syllable {
            onset: idx(ONSETS, onset),
            vowel: idx(VOWELS, vowel),
            vowel2: None,
            coda: coda.map(|c| idx(CODAS, c)),
        }
    }

    #[test]
    fn hangul_word_matches() {
        // gksrmf = 한글
        run_diff(&[
            syl('g', 'k', Some('s')),
            syl('r', 'm', Some('f')),
            DiffOp::Flush,
        ]);
    }

    #[test]
    fn compound_vowel_matches() {
        // rhkstjd = 관성 (ㄱㅗㅏㄴ / ㅅㅓㅇ)
        run_diff(&[
            DiffOp::Syllable {
                onset: idx(ONSETS, 'r'),
                vowel: idx(VOWELS, 'h'),
                vowel2: Some(idx(VOWELS, 'k')),
                coda: Some(idx(CODAS, 's')),
            },
            syl('t', 'j', Some('d')),
            DiffOp::Flush,
        ]);
    }

    #[test]
    fn jongseong_resplit_matches() {
        // dks + l = 아니 (jongseong ㄴ resplits into the next syllable)
        run_diff(&[syl('d', 'k', Some('s')), syl('d', 'l', None)]);
    }

    #[test]
    fn backspace_matches() {
        // type 한, delete past empty, keep typing
        let mut ops = vec![syl('g', 'k', Some('s'))];
        ops.extend([
            DiffOp::Backspace,
            DiffOp::Backspace,
            DiffOp::Backspace,
            DiffOp::Backspace,
        ]);
        ops.push(syl('r', 'm', Some('f')));
        run_diff(&ops);
    }
}
