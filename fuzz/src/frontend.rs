//! A Rust mirror of the GTK immodule's commit/emit/reset protocol driving
//! the REAL `InputEngine` against an arbitrary hostile client.
//!
//! Mirrored code: `src/frontends/gtk3/src/immodule.c` (compiled verbatim
//! into the gtk4 module too) — specifically `process_input_result`,
//! `kime_reset`, `update_preedit`, `commit` and the synchronous (GTK4)
//! shape of `filter_keypress`. The GTK3 build defers `update_preedit` to a
//! re-queued HANDLED_MASK event, but performs the same calls in the same
//! order, so the protocol under test is identical. The Qt frontend
//! (`src/frontends/qt5/src/input_context.cc`) implements the same
//! snapshot-and-clear-before-emit ordering, so this target guards both.
//!
//! The hostile client is what GTK apps really do from signal handlers
//! (kime#562: Firefox calls `gtk_im_context_reset()` from its "commit"
//! handler): every commit delivery and every preedit signal may synchronously
//! re-enter `reset()`.
//!
//! Oracle: the engine only ever APPENDS to its commit buffer (asserted), so
//! we can account for every produced character. Invariants, checked after
//! every top-level op:
//!   1. concat(delivered commits) + still-buffered == everything produced
//!      (each committed character reaches the client EXACTLY once, in order:
//!      a duplicate emit makes `delivered` too long, a swallowed commit makes
//!      it too short);
//!   2. commit-signal re-entrancy depth stays bounded.
//!
//! KNOWN WEAKNESS — this is a hand transcription, not the C code. It can
//! drift from the real frontends: whenever the commit/reset ordering in
//! `immodule.c` or `input_context.cc` changes, this file must be
//! re-transcribed by hand. Triage rule: reproduce every finding with the
//! e2e gtk/qt reset probes against the REAL frontends
//! (`tests/e2e`: `g3_06`/`g3_07`, `q06`/`q07`) before filing anything.
//! Not modeled: `commit_event` (direct English commit needs
//! gdk_keyval_to_unicode), NOT_READY/LANGUAGE_CHANGED side effects,
//! GObject ref-counting/finalize during emission.

use arbitrary::Arbitrary;
use kime_engine_core::{Config, InputEngine, InputResult, ModifierState};

use crate::presets;

#[derive(Arbitrary, Debug)]
pub enum FrontOp {
    Key {
        code: u8,
        state: u8,
        numlock: bool,
    },
    /// The app calls gtk_im_context_reset() between keys.
    Reset,
}

#[derive(Arbitrary, Debug)]
pub struct FrontendInput {
    pub preset: u8,
    /// bits 0/1/2: client calls reset() from inside its
    /// preedit-start/-changed/-end handler.
    pub preedit_reset_mask: u8,
    /// Per-commit hostile reactions: each commit the client receives consumes
    /// one byte; an odd byte means the handler synchronously calls
    /// gtk_im_context_reset() (the kime#562 Firefox pattern, at arbitrary
    /// depths and arbitrary points in the stream).
    pub reset_script: Vec<u8>,
    pub ops: Vec<FrontOp>,
}

const MAX_OPS: usize = 64;
/// A protocol-correct frontend never nests commit emission deeper than 2
/// even against a client that resets on every commit; generous margin.
const MAX_DEPTH: usize = 8;

struct Frontend<'c> {
    engine: InputEngine,
    config: &'c Config,
    /// mirror of ctx->buf
    buf: String,
    /// mirror of ctx->preedit_visible
    preedit_visible: bool,
    /// commit-signal nesting depth
    depth: usize,

    /// Test-only: run the PRE-#799 emit-then-clear ordering instead of the
    /// current immodule.c one. Must never be settable from the fuzz target.
    #[cfg(test)]
    emit_then_clear: bool,

    // ---- hostile client behavior ----
    reset_script: Vec<u8>,
    script_idx: usize,
    preedit_reset_mask: u8,

    // ---- oracle ----
    /// what we know the engine commit_buf currently holds
    known: String,
    /// every character the engine ever appended to commit_buf, in order
    produced: String,
    /// every character the client's commit handler received, in order
    delivered: String,
}

impl<'c> Frontend<'c> {
    fn new(input: &FrontendInput, config: &'c Config) -> Self {
        Frontend {
            engine: InputEngine::new(config),
            config,
            buf: String::new(),
            preedit_visible: false,
            depth: 0,
            #[cfg(test)]
            emit_then_clear: false,
            reset_script: input.reset_script.clone(),
            script_idx: 0,
            preedit_reset_mask: input.preedit_reset_mask & 0b111,
            known: String::new(),
            produced: String::new(),
            delivered: String::new(),
        }
    }

    /// Account for engine-side commit_buf growth. The engine treats
    /// commit_buf as append-only (only the frontend clears it), so anything
    /// beyond the known prefix was newly produced.
    fn observe_engine(&mut self) {
        let cur = self.engine.commit_str();
        assert!(
            cur.starts_with(self.known.as_str()),
            "engine commit_buf changed non-monotonically: known {:?} -> now {:?}",
            self.known,
            cur
        );
        let new_known = cur.to_owned();
        self.produced.push_str(&cur[self.known.len()..]);
        self.known = new_known;
    }

    /// str_buf_set_str(&ctx->buf, kime_engine_commit_str(ctx->engine))
    fn set_buf_from_commit_str(&mut self) {
        self.buf.clear();
        let s = self.engine.commit_str().to_owned();
        self.buf.push_str(&s);
    }

    /// immodule.c `commit()`: g_signal_emit "commit" with ctx->buf, then
    /// clear ctx->buf. The client handler runs INSIDE the emit and may
    /// re-enter kime_reset.
    fn emit_commit(&mut self) {
        if self.buf.is_empty() {
            return;
        }
        self.depth += 1;
        assert!(
            self.depth <= MAX_DEPTH,
            "commit signal re-entrancy exceeded depth {}: a reset-in-commit client \
             recurses without bound (delivered so far: {:?})",
            MAX_DEPTH,
            self.delivered
        );
        let text = std::mem::take(&mut self.buf);
        self.delivered.push_str(&text);
        // hostile client reaction
        if let Some(&b) = self.reset_script.get(self.script_idx) {
            self.script_idx += 1;
            if b & 1 == 1 {
                self.kime_reset();
            }
        }
        self.depth -= 1;
        // C: `ctx->buf.len = 0;` after g_signal_emit returns
        self.buf.clear();
    }

    /// immodule.c `process_input_result` (commit branch only; NOT_READY and
    /// LANGUAGE_CHANGED touch OS state and are not modeled).
    fn process_input_result(&mut self, ret: InputResult) -> (bool, bool) {
        let bypassed = !ret.contains(InputResult::CONSUMED);
        let has_preedit = ret.contains(InputResult::HAS_PREEDIT);

        if ret.contains(InputResult::HAS_COMMIT) {
            self.handle_commit();
        }

        (bypassed, has_preedit)
    }

    /// The HAS_COMMIT branch of `process_input_result`: snapshot the commit
    /// string and clear the engine buffer BEFORE emitting, so a re-entrant
    /// reset from the client's "commit" handler sees an empty engine.
    fn handle_commit(&mut self) {
        #[cfg(test)]
        if self.emit_then_clear {
            self.handle_commit_pre_fix();
            return;
        }
        self.set_buf_from_commit_str();
        self.engine.clear_commit();
        self.known.clear();
        self.emit_commit();
    }

    /// The PRE-#799 immodule.c ordering: read -> EMIT -> clear. The clear is
    /// too late — a reset re-entering from inside the emission re-reads the
    /// still-populated engine buffer and commits the same string again
    /// (kime#562). Kept only for the should_panic regression test.
    #[cfg(test)]
    fn handle_commit_pre_fix(&mut self) {
        self.set_buf_from_commit_str();
        self.emit_commit();
        self.engine.clear_commit();
        self.known.clear();
    }

    /// immodule.c `kime_reset` — reachable re-entrantly from the client's
    /// commit / preedit handlers via gtk_im_context_reset().
    fn kime_reset(&mut self) {
        // clear_preedit APPENDS the live preedit to the engine commit_buf
        // (src/engine/core/src/lib.rs clear_preedit)
        self.engine.clear_preedit();
        self.observe_engine();
        #[cfg(test)]
        if self.emit_then_clear {
            self.kime_reset_pre_fix_tail();
            return;
        }
        // immodule.c: snapshot, reset the engine, THEN emit
        self.set_buf_from_commit_str();
        self.engine.reset();
        self.known.clear();
        self.emit_commit();
    }

    /// The PRE-#799 `kime_reset` tail: emit first, reset the engine (and its
    /// commit_buf) only afterwards. Kept only for the should_panic test.
    #[cfg(test)]
    fn kime_reset_pre_fix_tail(&mut self) {
        self.set_buf_from_commit_str();
        self.emit_commit();
        self.engine.reset();
        self.known.clear();
    }

    /// immodule.c `update_preedit`
    fn update_preedit(&mut self) {
        let visible = !self.engine.preedit_str().is_empty();
        if self.preedit_visible != visible {
            self.preedit_visible = visible;
            if visible {
                self.client_preedit_signal(0); // preedit-start
                self.client_preedit_signal(1); // preedit-changed
            } else {
                self.client_preedit_signal(1); // preedit-changed
                self.client_preedit_signal(2); // preedit-end
            }
        } else if visible {
            self.client_preedit_signal(1); // preedit-changed
        }
    }

    fn client_preedit_signal(&mut self, which: u8) {
        if self.preedit_reset_mask & (1 << which) != 0 {
            self.kime_reset();
        }
    }

    /// immodule.c `filter_keypress`, GTK4 synchronous shape. The GTK3 build
    /// runs `on_key_input` now and `update_preedit` on the re-queued
    /// HANDLED_MASK event — same calls, same order, split across two events.
    fn filter_key(&mut self, code: u8, state: u8, numlock: bool) {
        let state = ModifierState::from_bits_truncate(state as u32);
        let ret = self
            .engine
            .press_key_code(code as u16, state, numlock, self.config);
        self.observe_engine();
        let (_bypassed, has_preedit) = self.process_input_result(ret);
        if self.preedit_visible || has_preedit {
            self.update_preedit();
        }
        // commit_event (direct English commit of bypassed keys) is
        // frontend-generated text needing gdk_keyval_to_unicode; not modeled.
    }

    /// Invariant 1, valid whenever no signal emission is in flight.
    fn check_quiescent(&self) {
        let pending = self.engine.commit_str();
        let mut expect = self.delivered.clone();
        expect.push_str(pending);
        assert_eq!(
            self.produced,
            expect,
            "commit protocol violated: engine produced {:?} but client received {:?} \
             (still buffered: {:?}) — commit text was {}",
            self.produced,
            self.delivered,
            pending,
            if expect.len() > self.produced.len() {
                "delivered MORE than once"
            } else {
                "LOST"
            }
        );
    }

    fn run(&mut self, ops: &[FrontOp]) {
        for op in ops.iter().take(MAX_OPS) {
            match *op {
                FrontOp::Key {
                    code,
                    state,
                    numlock,
                } => self.filter_key(code, state, numlock),
                FrontOp::Reset => self.kime_reset(),
            }
            self.check_quiescent();
        }

        // teardown = focus_out: one final kime_reset must flush, not lose
        self.kime_reset();
        self.check_quiescent();
        assert_eq!(
            self.produced, self.delivered,
            "after teardown reset some produced text never reached the client"
        );
    }
}

pub fn run_frontend(input: &FrontendInput) {
    let presets = presets();
    let config = &presets[input.preset as usize % presets.len()];
    Frontend::new(input, config).run(&input.ops);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The PRE-FIX transcription: emit the commit signal first, clear the
    /// engine buffer after. This is the #562/#799 bug class the fuzz target
    /// exists to guard against — kept only so the should_panic test below
    /// proves the oracle still catches it. The real-frontend guards are the
    /// e2e probes `g3_07_562_reset_in_commit_double` (gtk3.rs) and
    /// `q07_reset_in_commit_double` (qt.rs).
    fn run_frontend_pre_fix_emit_then_clear(input: &FrontendInput) {
        let presets = presets();
        let config = &presets[input.preset as usize % presets.len()];
        let mut f = Frontend::new(input, config);
        f.emit_then_clear = true;
        f.run(&input.ops);
    }

    fn dubeolsik_keys(codes: &[u8]) -> Vec<FrontOp> {
        codes
            .iter()
            .map(|&code| FrontOp::Key {
                code,
                state: 0,
                numlock: false,
            })
            .collect()
    }

    /// The kime#562 pattern: client resets inside its commit handler.
    /// X-style hardware codes (evdev+8): d=40 h=43 f=41 o=32 s=39 -> 오랜 (dhfos);
    /// the 오 commit fires mid-stream when ㅐ resplits the syllable.
    fn firefox_pattern() -> FrontendInput {
        FrontendInput {
            preset: 0,
            preedit_reset_mask: 0,
            reset_script: vec![1],
            ops: dubeolsik_keys(&[40, 43, 41, 32, 39]),
        }
    }

    #[test]
    fn survives_firefox_reset_in_commit_handler() {
        run_frontend(&firefox_pattern());
    }

    /// Documents the #562/#799 double-commit: with the pre-fix ordering in
    /// immodule.c the re-entrant reset re-read the uncleared engine buffer
    /// and delivered the same string twice. If the current ordering ever
    /// regresses to this, the fuzz target finds it in seconds — and the e2e
    /// guards g3_07/q07 catch it against the real frontends.
    #[test]
    #[should_panic(expected = "commit protocol violated")]
    fn pre_fix_emit_then_clear_double_commits_firefox_pattern() {
        run_frontend_pre_fix_emit_then_clear(&firefox_pattern());
    }

    #[test]
    fn survives_reset_on_every_commit_and_preedit_signal() {
        let input = FrontendInput {
            preset: 0,
            preedit_reset_mask: 0b111,
            reset_script: vec![1; 64],
            ops: dubeolsik_keys(&[40, 43, 41, 32, 39, 40, 43, 41, 32, 39]),
        };
        run_frontend(&input);
    }
}
