pub mod diff;
pub mod frontend;

use arbitrary::Arbitrary;
use kime_engine_config::{EngineConfig, HotkeyBehavior, InputCategory};
use kime_engine_core::{Config, InputEngine, InputResult, ModifierState};
use std::sync::LazyLock;

/// Preset configs, built once: `Config::new` loads system fonts via fontdb
/// (hundreds of ms), which must not run per fuzz input.
///
/// Mode hotkeys (Hanja/Emoji/Math) are stripped from every preset:
/// entering Hanja mode spawns the kime-candidate-window child process
/// (`HanjaMode::set_key` -> `Client::new`), which a fuzzer must never do.
/// Category Switch/Toggle hotkeys stay — category flips are part of the
/// state machine under test.
static PRESETS: LazyLock<Vec<Config>> = LazyLock::new(|| {
    let mut presets = Vec::new();
    for layout in ["dubeolsik", "sebeolsik-3-90", "sebeolsik-3-91"] {
        for word_commit in [false, true] {
            let mut engine = EngineConfig::default();
            engine.default_category = InputCategory::Hangul;
            // never dial the indicator socket
            engine.global_category_state = false;
            engine.hangul.layout = layout.into();
            engine.hangul.word_commit = word_commit;
            engine
                .global_hotkeys
                .retain(|_, h| !matches!(h.behavior(), HotkeyBehavior::Mode(_)));
            engine.category_hotkeys.clear();
            engine.mode_hotkeys.clear();
            presets.push(Config::new(engine));
        }
    }
    presets
});

pub fn presets() -> &'static [Config] {
    &PRESETS
}

#[derive(Arbitrary, Debug)]
pub enum Op {
    /// `u8` keeps the search inside the meaningful evdev range (all codes
    /// kime maps are < 256); unknown codes still hit the `None` arm of
    /// `from_hardware_code`, which is itself under test.
    PressKey {
        code: u8,
        state: u8,
        numlock: bool,
    },
    SetCategory {
        hangul: bool,
    },
    Reset,
    ClearCommit,
    ClearPreedit,
    RemovePreedit,
    ReadPreedit,
}

const MAX_OPS: usize = 256;

pub fn run_ops(preset_idx: u8, ops: &[Op]) {
    let config = &presets()[preset_idx as usize % presets().len()];
    let mut engine = InputEngine::new(config);

    for op in ops.iter().take(MAX_OPS) {
        match op {
            Op::PressKey {
                code,
                state,
                numlock,
            } => {
                let state = ModifierState::from_bits_truncate(*state as u32);
                let ret = engine.press_key_code(*code as u16, state, *numlock, config);

                // current_result() is the sole producer of these flags, so
                // both directions must hold on every return value.
                assert_eq!(
                    ret.contains(InputResult::HAS_COMMIT),
                    !engine.commit_str().is_empty(),
                    "HAS_COMMIT flag out of sync with commit_buf"
                );
                let preedit_empty = engine.preedit_str().is_empty();
                assert_eq!(
                    ret.contains(InputResult::HAS_PREEDIT),
                    !preedit_empty,
                    "HAS_PREEDIT flag out of sync with preedit_str"
                );

                // What every frontend does after reading a commit.
                if ret.contains(InputResult::HAS_COMMIT) {
                    engine.clear_commit();
                }
            }
            Op::SetCategory { hangul } => engine.set_input_category(if *hangul {
                InputCategory::Hangul
            } else {
                InputCategory::Latin
            }),
            Op::Reset => engine.reset(),
            Op::ClearCommit => engine.clear_commit(),
            Op::ClearPreedit => engine.clear_preedit(),
            Op::RemovePreedit => engine.remove_preedit(),
            Op::ReadPreedit => {
                // force the preedit rebuild path to run
                let _ = engine.preedit_str().len();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// gksrmf on dubeolsik must build presets, run, and violate nothing.
    #[test]
    fn dubeolsik_hangul_word() {
        // evdev: g=34 k=37 s=31 r=19 m=50 f=33
        let ops: Vec<Op> = [34u8, 37, 31, 19, 50, 33]
            .into_iter()
            .map(|code| Op::PressKey {
                code,
                state: 0,
                numlock: false,
            })
            .collect();
        // preset 0 = dubeolsik, word_commit=false
        run_ops(0, &ops);
    }
}
