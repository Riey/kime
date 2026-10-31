//! GTK3 immodule end-to-end tests (staged local `libim-kime.so` on Xvfb).
//!
//! Run: `cargo test -p kime-e2e --test gtk3 -- --ignored --test-threads=1`

use std::path::Path;
use std::time::Duration;

use kime_e2e::kime;
use kime_e2e::paths::{self, ScratchDir};
use kime_e2e::probes::{self, GtkProbeOpts, GtkResetProbe, GuiProbe, ImEvent, PROBE_TITLE};
use kime_e2e::proc;
use kime_e2e::stage::{self, Staged};
use kime_e2e::x11::XvfbSession;

const WAIT: Duration = Duration::from_secs(10);

/// Stage the local GTK3 immodule, write a Hangul config, and spawn the probe
/// on `x` with `extra_env` appended. The caller must still
/// `focus_window(PROBE_TITLE)` before typing.
fn spawn_probe(
    x: &XvfbSession,
    scratch: &ScratchDir,
    textview: bool,
    extra_env: &[(String, String)],
) -> (GuiProbe, Staged) {
    let staged = stage::stage_gtk3(scratch).expect("stage gtk3 immodule");
    let xdg = kime::write_config(scratch, kime::HANGUL_CONFIG).expect("write kime config");
    let mut env = staged.env.clone();
    env.push(("XDG_CONFIG_HOME".into(), xdg.display().to_string()));
    env.extend_from_slice(extra_env);
    let opts = GtkProbeOpts {
        gtk_major: 3,
        textview,
    };
    let probe = probes::spawn_gtk_probe_x11(x, &env, &opts, scratch).expect("start gtk3 probe");
    (probe, staged)
}

/// Number of IM events logged so far (marker for [`wait_new_preedit`]).
fn im_len(log: &Path) -> usize {
    probes::read_im_log(log).len()
}

/// Wait until at least one new IM event arrived after `since` and the latest
/// event is `Preedit(expected)`. Only usable with the Entry probe — GTK3
/// TextView has no `preedit-changed` signal.
fn wait_new_preedit(log: &Path, expected: &str, since: usize) {
    proc::wait_until(
        &format!(
            "preedit {expected:?} in {} (events: {:?})",
            log.display(),
            probes::read_im_log(log)
        ),
        WAIT,
        || {
            let events = probes::read_im_log(log);
            events.len() > since
                && matches!(events.last(), Some(ImEvent::Preedit(s)) if s == expected)
        },
    )
    .expect("preedit did not appear");
}

/// Assert the non-empty preedit stream contains `expected` in order.
fn assert_preedit_sequence(log: &Path, expected: &[&str]) {
    let seq: Vec<String> = probes::read_im_log(log)
        .into_iter()
        .filter_map(|e| match e {
            ImEvent::Preedit(s) if !s.is_empty() => Some(s),
            _ => None,
        })
        .collect();
    let mut iter = seq.iter();
    for want in expected {
        assert!(
            iter.any(|s| s == want),
            "preedit stream {seq:?} does not contain {expected:?} in order"
        );
    }
}

/// Live child pids of `pid` (all threads' `/proc/<pid>/task/*/children`).
/// Zombie children stay listed until reaped, which is exactly what the #617
/// test needs to observe.
fn child_pids(pid: i32) -> Vec<i32> {
    let mut out = Vec::new();
    if let Ok(entries) = std::fs::read_dir(format!("/proc/{pid}/task")) {
        for entry in entries.flatten() {
            if let Ok(s) = std::fs::read_to_string(entry.path().join("children")) {
                out.extend(s.split_whitespace().filter_map(|p| p.parse::<i32>().ok()));
            }
        }
    }
    out
}

/// `/proc/<pid>/comm` (kernel-truncated to 15 chars).
fn comm(pid: i32) -> String {
    std::fs::read_to_string(format!("/proc/{pid}/comm"))
        .map(|s| s.trim().to_string())
        .unwrap_or_default()
}

/// Process state char from `/proc/<pid>/stat` (`Z` = zombie/defunct).
fn stat_state(pid: i32) -> Option<char> {
    let stat = std::fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
    // Field 3 comes after the parenthesized comm, which may itself contain
    // spaces/parens — split at the LAST ')'.
    stat.rsplit(')')
        .next()?
        .split_whitespace()
        .next()?
        .chars()
        .next()
}

/// `kime-candidate-window` children of `pid` (comm is truncated to 15 chars,
/// so match on the prefix).
fn candidate_children(pid: i32) -> Vec<i32> {
    child_pids(pid)
        .into_iter()
        .filter(|&c| comm(c).starts_with("kime-candidate"))
        .collect()
}

/// G3-SMOKE: Xvfb → staged local `libim-kime.so` (private immodule cache via
/// `gtk-query-immodules-3.0` + `GTK_IM_MODULE_FILE`) → GTK3 Entry probe.
///
/// Types `gksrmf` + Enter (dubeolsik) and asserts the committed text `한글`
/// with the full preedit stream `ㅎ하한ㄱ그글`, plus a `/proc/<pid>/maps`
/// check that the LOCAL staged module loaded.
#[test]
#[ignore = "e2e: spawns Xvfb and a GTK3 app; run with --ignored"]
fn g3_smoke() {
    let scratch = ScratchDir::new("g3_smoke");
    let x = XvfbSession::new(&scratch).expect("start Xvfb");
    let (probe, staged) = spawn_probe(&x, &scratch, false, &[]);
    x.focus_window(PROBE_TITLE).expect("focus probe window");

    x.type_text("gksrmf").expect("type gksrmf");
    x.key("Return").expect("press Return");

    probe.buffer.wait_for("한글", WAIT).expect("committed 한글");
    stage::maps_check_staged(probe.pid(), &staged).expect("local gtk3 immodule loaded");
    assert_preedit_sequence(&probe.preedit_log, &["ㅎ", "하", "한", "ㄱ", "그", "글"]);
}

/// G3-01 (PR #775 GTK3 guard): the GTK3 deferred re-injection path is
/// unchanged — the same input as G4-01 yields the identical buffer.
///
/// PR #775 fixed the GTK4 build synchronously but intentionally kept GTK3's
/// `gdk_event_put` deferral (workaround for #536/#565/#570 app quirks); this
/// pins that behavior: `dks` + one Return → `안\n`, twice → `안\n안\n`.
/// GTK3 TextView has no `preedit-changed` signal, so this asserts on the
/// buffer only, with a short settle pause after typing.
#[test]
#[ignore = "e2e: spawns Xvfb and a GTK3 app; run with --ignored"]
fn g3_01_775_deferral_path_unchanged() {
    let scratch = ScratchDir::new("g3_01_775");
    let x = XvfbSession::new(&scratch).expect("start Xvfb");
    let (probe, staged) = spawn_probe(&x, &scratch, true, &[]);
    x.focus_window(PROBE_TITLE).expect("focus probe window");

    x.type_text("dks").expect("type dks");
    // No preedit-changed on GTK3 TextView; xdotool paces keys at 100ms and the
    // engine runs in-process, so a short settle is enough for preedit 안.
    proc::sleep_ms(500);
    stage::maps_check_staged(probe.pid(), &staged).expect("local gtk3 immodule loaded");
    x.key("Return").expect("press Return");
    probe
        .buffer
        .wait_for("안\n", WAIT)
        .expect("single Enter must commit 안 and insert the newline (GTK3 deferral)");

    x.type_text("dks").expect("type dks");
    proc::sleep_ms(500);
    x.key("Return").expect("press Return");
    probe
        .buffer
        .wait_for("안\n안\n", WAIT)
        .expect("second dks+Enter round must yield 안\\n안\\n");
}

/// G3-02 (#617 zombie half, PR #769 fix 2): dismissed hanja candidate popups
/// leave no `<defunct>` `kime-candidate-window` children behind.
///
/// The engine runs in-process in the probe, so the candidate window spawns as
/// a child of the probe pid. Loop 3×: `gks` → preedit 한 → Control_R opens
/// the popup (default `category_hotkeys` config) → BackSpace at the probe
/// cancels hanja mode via `HanjaMode::press_key` → `reset`, running
/// `Client::close`'s kill+reap path (`src/engine/candidate/src/client.rs`).
/// Pre-fix, `close` killed the child but never reaped it, accumulating one
/// zombie per dismissed popup (#617).
///
/// Esc is deliberately NOT used to dismiss here: in Hanja mode Esc is a
/// merged global hotkey and takes the `set_input_category` path — which is
/// #780's leak, guarded separately by [`g3_02b_780_global_hotkey_closes_candidate`].
/// Any non-hotkey key reaches `HanjaMode::press_key` and the close path under
/// test here.
#[test]
#[ignore = "e2e: spawns Xvfb, a GTK3 app, and kime-candidate-window; run with --ignored"]
fn g3_02_617_no_zombie_candidates() {
    let scratch = ScratchDir::new("g3_02_617");
    let x = XvfbSession::new(&scratch).expect("start Xvfb");
    // The candidate window is eframe/egui (glow) → software GL; the engine
    // spawns `kime-candidate-window` via PATH, so put the local target dir
    // first (system kime is installed on dev machines).
    let extra_env = vec![
        ("LIBGL_ALWAYS_SOFTWARE".into(), "1".into()),
        (
            "PATH".into(),
            format!(
                "{}:{}",
                paths::target_dir().display(),
                std::env::var("PATH").unwrap_or_default()
            ),
        ),
    ];
    let (probe, staged) = spawn_probe(&x, &scratch, false, &extra_env);
    x.focus_window(PROBE_TITLE).expect("focus probe window");

    for round in 1..=3 {
        // The popup may have taken focus in a previous round — refocus.
        x.focus_window(PROBE_TITLE).expect("focus probe window");
        let since = im_len(&probe.preedit_log);
        x.type_text("gks").expect("type gks");
        wait_new_preedit(&probe.preedit_log, "한", since);
        if round == 1 {
            stage::maps_check_staged(probe.pid(), &staged).expect("local gtk3 immodule loaded");
        }

        x.key("Control_R").expect("press Control_R");
        let mut child = 0;
        proc::wait_until(
            &format!("kime-candidate-window child of probe (round {round})"),
            Duration::from_secs(15),
            || match candidate_children(probe.pid()).first() {
                Some(&c) => {
                    child = c;
                    true
                }
                None => false,
            },
        )
        .expect("hanja candidate window did not spawn");

        // BackSpace at the probe: HanjaMode::press_key → reset → Client::close
        // (kill + reap). A zombie child stays listed in /proc children until
        // reaped, so this wait times out on the pre-#769 kill-without-wait.
        x.key("BackSpace").expect("press BackSpace");
        proc::wait_until(
            &format!("candidate child {child} to be closed AND reaped (round {round})"),
            WAIT,
            || !child_pids(probe.pid()).contains(&child),
        )
        .expect("candidate window child never reaped — #617 zombie regression");
    }

    let leftovers = candidate_children(probe.pid());
    assert!(
        leftovers.is_empty(),
        "candidate window children survived the loop: {leftovers:?}"
    );
    let zombies: Vec<i32> = child_pids(probe.pid())
        .into_iter()
        .filter(|&c| stat_state(c) == Some('Z'))
        .collect();
    assert!(
        zombies.is_empty(),
        "#617 regression: defunct children of the probe remain: {zombies:?}"
    );
}

/// G3-02b (#780, fix: #787): a GLOBAL hotkey firing while the hanja popup is
/// open must close the popup, not leak it.
///
/// Global hotkeys are merged into `mode_hotkeys[Hanja]`, so Esc in hanja mode
/// matches `!Switch Latin` and lands in `Engine::set_input_category` — which
/// cleared `mode` without dispatching the active mode's `reset()`, leaving
/// `hanja_mode.client` owning a popup nothing could ever close: the process
/// stayed alive indefinitely and became a permanently unreaped zombie once it
/// exited. FAILS on develop until #787 (`leave_mode()`) merges.
#[test]
#[ignore = "e2e: spawns Xvfb, a GTK3 app, and kime-candidate-window; run with --ignored"]
fn g3_02b_780_global_hotkey_closes_candidate() {
    let scratch = ScratchDir::new("g3_02b_780");
    let x = XvfbSession::new(&scratch).expect("start Xvfb");
    let extra_env = vec![
        ("LIBGL_ALWAYS_SOFTWARE".into(), "1".into()),
        (
            "PATH".into(),
            format!(
                "{}:{}",
                paths::target_dir().display(),
                std::env::var("PATH").unwrap_or_default()
            ),
        ),
    ];
    let (probe, staged) = spawn_probe(&x, &scratch, false, &extra_env);
    x.focus_window(PROBE_TITLE).expect("focus probe window");

    let since = im_len(&probe.preedit_log);
    x.type_text("gks").expect("type gks");
    wait_new_preedit(&probe.preedit_log, "한", since);
    stage::maps_check_staged(probe.pid(), &staged).expect("local gtk3 immodule loaded");

    x.key("Control_R").expect("press Control_R");
    let mut child = 0;
    proc::wait_until(
        "kime-candidate-window child of probe",
        Duration::from_secs(15),
        || match candidate_children(probe.pid()).first() {
            Some(&c) => {
                child = c;
                true
            }
            None => false,
        },
    )
    .expect("hanja candidate window did not spawn");

    // Esc: merged global hotkey `!Switch Latin` → Engine::set_input_category.
    // With the #787 fix the active mode's reset() runs first (Client::close
    // kill+reap); pre-fix the popup outlives this wait.
    x.focus_window(PROBE_TITLE).expect("refocus probe window");
    x.key("Escape").expect("press Escape");
    proc::wait_until(
        &format!("candidate child {child} to be closed AND reaped after Esc"),
        WAIT,
        || !child_pids(probe.pid()).contains(&child),
    )
    .expect("popup leaked after a global hotkey in hanja mode (#780; fails until #787 merges)");

    // The hotkey must still have done its job: category switched to Latin.
    x.type_text("d").expect("type d");
    probe
        .buffer
        .wait_contains("d", WAIT)
        .expect("Esc did not switch the category to Latin (expected a literal 'd')");
}

/// Stage the local GTK3 immodule, write a Hangul config, and spawn the
/// re-entrant-reset probe (a direct `Gtk.IMMulticontext` — Entry/TextView
/// hide theirs). The caller must still `focus_window(PROBE_TITLE)` before
/// typing.
fn spawn_reset_probe(
    x: &XvfbSession,
    scratch: &ScratchDir,
    reset_in_commit: bool,
) -> (GtkResetProbe, Staged) {
    let staged = stage::stage_gtk3(scratch).expect("stage gtk3 immodule");
    let xdg = kime::write_config(scratch, kime::HANGUL_CONFIG).expect("write kime config");
    let mut env = staged.env.clone();
    env.push(("XDG_CONFIG_HOME".into(), xdg.display().to_string()));
    let probe = probes::spawn_gtk_reset_probe_x11(x, &env, reset_in_commit, scratch)
        .expect("start gtk3 reset probe");
    (probe, staged)
}

/// Wait until the reset probe logged a commit line containing `한`, then let
/// the event loop settle: the duplicate (if any) is emitted synchronously by
/// the nested handler, but the GTK3 immodule also re-queues the event
/// (HANDLED_MASK pass), so give any straggler time to land before reading.
fn wait_commits_settled(probe: &GtkResetProbe) -> Vec<String> {
    proc::wait_until(
        &format!(
            "a 한 commit line in {} (lines: {:?})",
            probe.commits_log.display(),
            probe.commit_lines()
        ),
        WAIT,
        || probe.commit_lines().iter().any(|l| l.contains("한")),
    )
    .expect("no commit ever arrived");
    proc::sleep_ms(500);
    probe.commit_lines()
}

/// G3-06 (guards the probe for G3-07): with a do-nothing commit handler,
/// `gks` + Return through the staged immodule delivers exactly ONE `한`
/// commit at depth 1. If this fails, the probe/staging is broken and G3-07
/// proves nothing.
#[test]
#[ignore = "e2e: spawns Xvfb and a GTK3 app; run with --ignored"]
fn g3_06_reset_in_commit_baseline() {
    let scratch = ScratchDir::new("g3_06_reset_base");
    let x = XvfbSession::new(&scratch).expect("start Xvfb");
    let (probe, staged) = spawn_reset_probe(&x, &scratch, false);
    x.focus_window(PROBE_TITLE).expect("focus probe window");

    x.type_text("gks").expect("type gks");
    x.key("Return").expect("press Return");

    let lines = wait_commits_settled(&probe);
    stage::maps_check_staged(probe.pid(), &staged).expect("local gtk3 immodule loaded");
    assert!(
        lines == ["d1:한"],
        "baseline: one keypress must commit 한 exactly once at depth 1;\n.commits was: {lines:?}"
    );
}

/// G3-07 (#562; guard added by #563, removed by #570 — RED until fixed):
/// a client calling `gtk_im_context_reset()` from inside its "commit"
/// handler receives the SAME text twice.
///
/// `process_input_result` (src/frontends/gtk3/src/immodule.c) emits the
/// commit signal BEFORE `kime_engine_clear_commit`, and `reset` →
/// `kime_reset` is unguarded: the handler's `reset()` re-enters, re-reads
/// the still-uncleared engine commit buffer, and emits `한` again from the
/// nested emission (`d2:한`). That is the kime#562 Firefox pattern (reset
/// from the commit path); #563 fixed it with an `is_committing` guard and
/// #570 (753b106) dropped the guard. A client that resets on EVERY commit
/// recurses without bound — the probe caps its reset at depth 1, so the bug
/// shows as exactly one duplicate.
#[test]
#[ignore = "e2e: spawns Xvfb and a GTK3 app; run with --ignored"]
fn g3_07_562_reset_in_commit_double() {
    let scratch = ScratchDir::new("g3_07_562");
    let x = XvfbSession::new(&scratch).expect("start Xvfb");
    let (probe, staged) = spawn_reset_probe(&x, &scratch, true);
    x.focus_window(PROBE_TITLE).expect("focus probe window");

    x.type_text("gks").expect("type gks");
    x.key("Return").expect("press Return");

    let lines = wait_commits_settled(&probe);
    stage::maps_check_staged(probe.pid(), &staged).expect("local gtk3 immodule loaded");
    assert!(
        lines == ["d1:한"],
        "#562 regression: reset() inside the commit handler re-delivered the \
         commit (immodule.c emits before kime_engine_clear_commit and \
         kime_reset is unguarded since #570);\n.commits was: {lines:?}"
    );
}
