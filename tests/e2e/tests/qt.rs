//! Qt frontend end-to-end tests (staged Qt5/Qt6 input-context plugin, engine
//! in-process).
//!
//! Run: `cargo test -p kime-e2e --test qt -- --ignored --test-threads=1`

use std::time::Duration;

use kime_e2e::inject::{key, VkbdInjector};
use kime_e2e::kime;
use kime_e2e::paths::{self, ScratchDir};
use kime_e2e::probes::{self, ImEvent};
use kime_e2e::proc;
use kime_e2e::stage;
use kime_e2e::sway::SwaySession;
use kime_e2e::x11::XvfbSession;

/// Children of `pid` whose cmdline is `kime-candidate-window`.
///
/// Matches on `/proc/<child>/cmdline` (not `comm`, which truncates the name
/// to 15 bytes).
fn candidate_children(pid: i32) -> Vec<i32> {
    let mut out = Vec::new();
    let Ok(tasks) = std::fs::read_dir(format!("/proc/{pid}/task")) else {
        return out;
    };
    for task in tasks.flatten() {
        let Ok(children) = std::fs::read_to_string(task.path().join("children")) else {
            continue;
        };
        for tok in children.split_whitespace() {
            let Ok(child) = tok.parse::<i32>() else {
                continue;
            };
            let cmdline = std::fs::read(format!("/proc/{child}/cmdline")).unwrap_or_default();
            if String::from_utf8_lossy(&cmdline).contains("kime-candidate-window") {
                out.push(child);
            }
        }
    }
    out
}

/// True when the plugin's embedded metadata carries the version-suffixed
/// input-context factory IID (`...QPlatformInputContextFactoryInterface.5.1`)
/// that Qt requires to load it. A bare unsuffixed IID means the build recipe
/// dropped the `KIME_QT_IID` define (#736/#756 bug class).
fn plugin_has_versioned_iid(module: &std::path::Path) -> bool {
    let bytes = std::fs::read(module).expect("read staged qt plugin");
    bytes
        .windows(b"QPlatformInputContextFactoryInterface.5.1".len())
        .any(|w| w == b"QPlatformInputContextFactoryInterface.5.1")
}

/// Focus a window by title, retrying until it is viewable — `windowfocus`
/// fails with BadMatch while the window exists but is not yet mapped (the
/// eframe candidate popup takes a moment between fork and map).
fn focus_window_retry(
    x: &XvfbSession,
    pattern: &str,
    timeout: Duration,
) -> kime_e2e::Result<String> {
    let mut wid = String::new();
    proc::wait_until(&format!("focusable window {pattern:?}"), timeout, || {
        let Ok(out) = x.xdotool(&["search", "--name", pattern]) else {
            return false;
        };
        let Some(w) = out.lines().next().filter(|w| !w.is_empty()) else {
            return false;
        };
        if x.xdotool(&["windowfocus", "--sync", w]).is_ok() {
            wid = w.to_string();
            true
        } else {
            false
        }
    })?;
    Ok(wid)
}

/// Shared smoke body: Xvfb + staged plugin + Qt probe; `gksrmf` + Enter must
/// commit `한글` through the kime input context.
///
/// This is the regression test for #736/#756 (Qt6 plugin metadata IID broken
/// twice after the meson migration): pre-fix Qt silently loads *no* input
/// method, so the buffer receives the literal `gksrmf` and the probe logs no
/// `QInputMethodEvent` at all. The `/proc/maps` check additionally proves the
/// freshly built plugin loaded rather than a stale system copy.
fn qt_smoke(qt_major: u32) {
    if qt_major == 5 && paths::immodule_opt(paths::Frontend::Qt5).is_none() {
        // Not a regression: a build may leave the qt5 plugin out (meson
        // `qt5=auto` finds no Qt5, as in the nix devshell CI uses — nixpkgs
        // refuses Qt5 and Qt6 side by side). The qt6 smoke below never takes
        // this path: its plugin is required and its absence must fail.
        eprintln!(
            "SKIP q5_smoke: no qt5 plugin in {} — this build has the qt5 frontend disabled",
            paths::build_dir().display()
        );
        return;
    }
    let scratch = ScratchDir::new(&format!("qt{qt_major}_smoke"));
    let x = XvfbSession::new(&scratch).expect("start Xvfb");
    let xdg = kime::write_config(&scratch, kime::HANGUL_CONFIG).expect("write kime config");
    let staged = stage::stage_qt(&scratch, qt_major).expect("stage qt plugin");
    // FAILS on develop until #778 (fix: #785) merges for qt5: the meson qt5
    // recipe drops the KIME_QT_IID define, the metadata IID lacks `.5.1`, and
    // Qt5 silently loads no input context.
    assert!(
        plugin_has_versioned_iid(&staged.module_path),
        "qt{qt_major} plugin metadata IID lacks '.5.1' — KIME_QT_IID define missing from the \
         meson recipe, Qt loads no kime input context (#736/#756 bug class; qt5: #778)"
    );
    let mut env = staged.env.clone();
    env.push(("XDG_CONFIG_HOME".into(), xdg.display().to_string()));
    let mut probe =
        probes::spawn_qt_probe_x11(&x, qt_major, &env, &scratch).expect("start qt probe");
    stage::maps_check_staged(probe.pid(), &staged).expect("staged kime plugin loaded");
    x.focus_window(probes::PROBE_TITLE).expect("focus probe");

    x.type_text("gksrmf").expect("type gksrmf");
    x.key("Return").expect("press Return");

    probe
        .buffer
        .wait_for("한글", Duration::from_secs(10))
        .expect("committed 한글 (a literal 'gksrmf' here means Qt loaded no IM — #736/#756)");
    let events = probes::read_im_log(&probe.preedit_log);
    assert!(
        events
            .iter()
            .any(|e| matches!(e, ImEvent::Preedit(p) if !p.is_empty())),
        "no preedit QInputMethodEvent observed — kime input context inactive (#736/#756); events: {events:?}"
    );
    assert!(probe.alive(), "qt{qt_major} probe died during the test");
}

/// Q6-SMOKE — regression test for #736/#756 (Qt6 input-context plugin failed
/// to load at all: IID mismatch, then IID define not passed to moc).
#[test]
#[ignore = "e2e: spawns Xvfb and a Qt6 GUI probe; run with --ignored"]
fn q6_smoke() {
    qt_smoke(6);
}

/// Q5-SMOKE — same plugin-loading chain for the Qt5 build.
#[test]
#[ignore = "e2e: spawns Xvfb and a Qt5 GUI probe; run with --ignored"]
fn q5_smoke() {
    qt_smoke(5);
}

/// Q-01 — issue #757 (refs #695/#617/#545), fixed by PR #771: the hanja
/// candidate popup must survive the focus transfer it itself causes, and the
/// syllable being converted must not be discarded.
///
/// Pre-fix, losing focus to the just-spawned `kime-candidate-window` ran
/// `reset()` in the Qt input context, which killed the popup instantly and
/// discarded the preedit. The fix (`src/frontends/qt5/src/input_context.cc`,
/// shared by the qt5 and qt6 builds) marks the engine NOT_READY so
/// `setFocusObject(nullptr)` skips the reset.
///
/// FAILS on develop until #779 (fix: #784) merges: PR #771 left the
/// `commit()`-on-focus-out path unguarded — Qt calls `commit()` before
/// `setFocusObject(nullptr)`, its unconditional `reset()` kills the popup,
/// and the `engine_ready` guard never runs.
#[test]
#[ignore = "e2e: spawns Xvfb, a Qt6 GUI probe and kime-candidate-window; run with --ignored"]
fn q01_757_candidate_survives_focus_loss() {
    let scratch = ScratchDir::new("q01_757");
    let x = XvfbSession::new(&scratch).expect("start Xvfb");
    let xdg = kime::write_config(&scratch, kime::HANGUL_CONFIG).expect("write kime config");
    let staged = stage::stage_qt(&scratch, 6).expect("stage qt6 plugin");
    let mut env = staged.env.clone();
    env.push(("XDG_CONFIG_HOME".into(), xdg.display().to_string()));
    // kime-candidate-window is eframe/egui-glow: needs software GL on Xvfb.
    env.push(("LIBGL_ALWAYS_SOFTWARE".into(), "1".into()));
    // The engine spawns `kime-candidate-window` via PATH lookup; prepend the
    // local target dir so the freshly built binary runs, not a system copy.
    env.push((
        "PATH".into(),
        format!(
            "{}:{}",
            paths::target_dir().display(),
            std::env::var("PATH").unwrap_or_default()
        ),
    ));
    let mut probe = probes::spawn_qt_probe_x11(&x, 6, &env, &scratch).expect("start qt6 probe");
    stage::maps_check_staged(probe.pid(), &staged).expect("staged kime plugin loaded");
    x.focus_window(probes::PROBE_TITLE).expect("focus probe");

    x.type_text("gks").expect("type gks");
    proc::wait_until(
        "preedit 한 in the QInputMethodEvent log",
        Duration::from_secs(10),
        || {
            probes::read_im_log(&probe.preedit_log)
                .iter()
                .any(|e| *e == ImEvent::Preedit("한".into()))
        },
    )
    .expect("preedit 한");

    x.key("Control_R").expect("press Control_R (hanja mode)");
    let mut popup: i32 = 0;
    proc::wait_until(
        "kime-candidate-window child of the probe",
        Duration::from_secs(15),
        || match candidate_children(probe.pid()).first() {
            Some(&pid) => {
                popup = pid;
                true
            }
            None => false,
        },
    )
    .expect("candidate window spawned");

    // Entering hanja mode legitimately emits one empty-preedit event (the
    // syllable moved into the popup); settle, then mark the log so the
    // assertion below only sees events caused by the focus change.
    proc::sleep_ms(500);
    let marker = probes::read_im_log(&probe.preedit_log).len();

    // Emulate a WM handing focus to the popup (Xvfb has no WM): FocusOut on
    // the probe drives Qt's setFocusObject(nullptr) — the exact #757 trigger.
    focus_window_retry(&x, "kime-candidate", Duration::from_secs(15))
        .expect("focus the candidate popup");
    proc::sleep_ms(1000);

    assert!(
        proc::pid_alive(popup),
        "candidate window (pid {popup}) died after the app lost focus — Qt's focus-out \
         commit() runs reset() before setFocusObject's engine_ready guard (#757 residual, \
         #779; fails until #784 merges)"
    );
    let events = probes::read_im_log(&probe.preedit_log);
    let after = &events[marker.min(events.len())..];
    assert!(
        !after.contains(&ImEvent::Preedit(String::new())),
        "preedit was discarded on focus loss — #757 regression; events after hanja entry: {after:?}"
    );
    assert!(probe.alive(), "qt6 probe died during the test");

    // Teardown: the popup still has focus — Escape makes it close itself.
    // Belt and braces: SIGKILL by exact pid (no-op if already gone).
    let _ = x.key("Escape");
    proc::sleep_ms(300);
    unsafe {
        libc::kill(popup, libc::SIGKILL);
    }
}

/// `d<depth>:<text>` lines from the qt probe's `.commits` log (written by its
/// `KIME_PROBE_RESET_IN_COMMIT` inputMethodEvent override; missing file =
/// none yet).
fn commit_depth_lines(probe: &kime_e2e::probes::GuiProbe) -> Vec<String> {
    let path = probe.buffer.path().with_extension("txt.commits");
    let Ok(bytes) = std::fs::read(path) else {
        return Vec::new();
    };
    String::from_utf8_lossy(&bytes)
        .lines()
        .map(str::to_string)
        .collect()
}

/// Shared Q-06/Q-07 body: Xvfb + staged qt6 plugin + the probe (with the
/// reset-in-commit override when asked), type `gks` + Return, wait for a 한
/// commit `QInputMethodEvent`, then let the event loop settle — the
/// re-entrant duplicate (if any) arrives synchronously inside the outer
/// delivery, but give any straggler time to land before reading. Returns
/// (IM events, `.commits` depth lines).
fn qt_reset_round(scratch_name: &str, reset_in_commit: bool) -> (Vec<ImEvent>, Vec<String>) {
    let scratch = ScratchDir::new(scratch_name);
    let x = XvfbSession::new(&scratch).expect("start Xvfb");
    let xdg = kime::write_config(&scratch, kime::HANGUL_CONFIG).expect("write kime config");
    let staged = stage::stage_qt(&scratch, 6).expect("stage qt6 plugin");
    let mut env = staged.env.clone();
    env.push(("XDG_CONFIG_HOME".into(), xdg.display().to_string()));
    if reset_in_commit {
        env.push(("KIME_PROBE_RESET_IN_COMMIT".into(), "1".into()));
    }
    let mut probe = probes::spawn_qt_probe_x11(&x, 6, &env, &scratch).expect("start qt6 probe");
    stage::maps_check_staged(probe.pid(), &staged).expect("staged kime plugin loaded");
    x.focus_window(probes::PROBE_TITLE).expect("focus probe");

    x.type_text("gks").expect("type gks");
    x.key("Return").expect("press Return");

    proc::wait_until(
        &format!(
            "a 한 commit event in {} (events: {:?})",
            probe.preedit_log.display(),
            probes::read_im_log(&probe.preedit_log)
        ),
        Duration::from_secs(10),
        || {
            probes::read_im_log(&probe.preedit_log)
                .iter()
                .any(|e| matches!(e, ImEvent::Commit(c) if c.contains("한")))
        },
    )
    .expect("no commit ever arrived");
    proc::sleep_ms(500);
    assert!(probe.alive(), "qt6 probe died during the test");
    let events = probes::read_im_log(&probe.preedit_log);
    let depth_lines = commit_depth_lines(&probe);
    (events, depth_lines)
}

/// Q-06 (guards the probe for Q-07): with the reset-in-commit override
/// disabled, `gks` + Return through the staged qt6 plugin delivers exactly
/// ONE 한 commit `QInputMethodEvent`. If this fails, the probe/staging is
/// broken and Q-07 proves nothing.
#[test]
#[ignore = "e2e: spawns Xvfb and a Qt6 GUI probe; run with --ignored"]
fn q06_reset_in_commit_baseline() {
    let (events, depth_lines) = qt_reset_round("q06_reset_base", false);
    let commits: Vec<String> = events
        .iter()
        .filter_map(|e| match e {
            ImEvent::Commit(c) => Some(c.clone()),
            _ => None,
        })
        .collect();
    assert!(
        commits == ["한"],
        "baseline: one keypress must commit 한 exactly once;\ncommits: {commits:?}\nevents: {events:?}"
    );
    assert!(
        depth_lines.is_empty(),
        "normal mode must not write the .commits log (probe mode leaked);\ngot: {depth_lines:?}"
    );
}

/// Q-07 (#562-class — the qt twin of G3-07): a client whose
/// `inputMethodEvent()` calls `QGuiApplication::inputMethod()->reset()` upon
/// a commit receives the SAME text twice.
///
/// `process_input_result` (src/frontends/qt5/src/input_context.cc, shared by
/// the qt5 and qt6 builds) emits the commit via the SYNCHRONOUS
/// `QCoreApplication::sendEvent` BEFORE `kime_engine_clear_commit`, and
/// `reset()` is unguarded (`clear_preedit` → `commit_str(emit)` →
/// `kime_engine_reset`): the widget's reset re-enters while the outer
/// sendEvent is still on the stack, re-reads the still-uncleared engine
/// commit buffer, and delivers 한 again (`d2:한`). The probe caps its reset
/// at depth 1 — an always-resetting client would recurse without bound — so
/// the bug shows as exactly one duplicate line.
#[test]
#[ignore = "e2e: spawns Xvfb and a Qt6 GUI probe; run with --ignored"]
fn q07_reset_in_commit_double() {
    let (events, depth_lines) = qt_reset_round("q07_reset_double", true);
    assert!(
        depth_lines == ["d1:한"],
        "#562-class regression: reset() inside inputMethodEvent re-delivered \
         the commit (input_context.cc emits via sendEvent before \
         kime_engine_clear_commit and reset() is unguarded);\n.commits was: \
         {depth_lines:?}\nevents: {events:?}"
    );
}

/// True when the Qt6 wayland platform plugin appears to be installed. When the
/// platforms dir is missing entirely we optimistically return true and let the
/// probe spawn decide.
fn qt6_wayland_available() -> bool {
    let dir = std::path::Path::new("/usr/lib/qt6/plugins/platforms");
    let Ok(entries) = std::fs::read_dir(dir) else {
        return true;
    };
    entries
        .flatten()
        .any(|e| e.file_name().to_string_lossy().starts_with("libqwayland"))
}

/// Config binding ONLY `M-AltR` (AltR with its own ALT bit) to the toggle.
/// The toggle can then only fire when the frontend delivers `Key{AltR, ALT}`
/// — an exact-match detector for the #760 event shape.
const DETECTOR_CONFIG: &str = "engine:
  default_category: Latin
  global_hotkeys:
    M-AltR:
      behavior: !Toggle [Hangul, Latin]
      result: Consume
";

/// One AltR-toggle round through a Qt6 wayland probe: baseline `r`, lone
/// AltR, then `rk` + Enter. Returns the final buffer (`"r가"` when the toggle
/// fired, `"rrk"` when it did not).
fn altr_round(
    sway: &SwaySession,
    inject: &mut VkbdInjector,
    staged: &kime_e2e::stage::Staged,
    xdg: &std::path::Path,
    scratch: &ScratchDir,
) -> String {
    let mut env = staged.env.clone();
    env.push(("XDG_CONFIG_HOME".into(), xdg.display().to_string()));
    // WAYLAND_DEBUG gives us the wl_keyboard.enter line for focus sync.
    env.push(("WAYLAND_DEBUG".into(), "1".into()));
    let mut probe =
        probes::spawn_qt_probe_wayland(sway.socket(), 6, &env, scratch).expect("start qt6 probe");
    stage::maps_check_staged(probe.pid(), staged).expect("staged kime plugin loaded");
    // No kime-wayland in this test: injected keys reach the probe directly,
    // but only once sway focuses its surface.
    proc::wait_for_line(
        &probe.log,
        &["wl_keyboard", ".enter("],
        Duration::from_secs(10),
    )
    .expect("probe got keyboard focus");

    // Latin baseline: 'r' passes through.
    inject.tap(19).expect("tap r");
    probe
        .buffer
        .wait_for("r", Duration::from_secs(10))
        .expect("Latin baseline 'r'");

    inject.press(key::RIGHTALT).expect("press AltR");
    inject.release(key::RIGHTALT).expect("release AltR");
    // If the toggle fired: rk → preedit 가, Enter commits. If not: literal.
    inject.tap_seq(&[19, 37]).expect("tap rk");
    inject.tap(key::ENTER).expect("tap Enter");
    proc::wait_until(
        &format!(
            "buffer settles to r가 or rrk (last: {:?})",
            probe.buffer.read()
        ),
        Duration::from_secs(10),
        || matches!(probe.buffer.read().as_str(), "r가" | "rrk"),
    )
    .expect("buffer settled");
    assert!(probe.alive(), "qt6 probe died during the test");
    let text = probe.buffer.read();
    // The next round reuses the same dump-file names: drop the probe first,
    // then remove its files so stale content can't satisfy the next waits.
    drop(probe);
    let _ = std::fs::remove_file(scratch.file("qt6-buffer.txt"));
    let _ = std::fs::remove_file(scratch.file("qt6-buffer.txt.preedit"));
    text
}

/// Q-02 — issue #760, fixed by PR #760: an AltR hangul toggle must fire even
/// when the press event carries its own modifier bit (`Key { AltR, ALT }`),
/// as Wayland-native toolkits report post-event modifier state.
///
/// Setup: headless sway → virtual-keyboard injector → Qt6 probe with
/// `QT_QPA_PLATFORM=wayland` + the staged kime plugin (engine in-process; no
/// kime-wayland).
///
/// Phase 1 (detector): a config binding only `M-AltR` can toggle only when
/// the frontend delivered `Key{AltR, ALT}` — the exact #760 shape. When the
/// event stream of this setup does not produce the shape, the test skips
/// cleanly (the matching logic is unit-tested in
/// `src/engine/core/tests/self_modifier.rs`).
///
/// Phase 2 (regression): the stock plain-`AltR` binding must fire on that
/// same event shape. Pre-fix the exact hotkey match missed `Key{AltR, ALT}`,
/// the toggle never fired, and the buffer ended up as literal `rrk`.
#[test]
#[ignore = "e2e: spawns headless sway and Qt6 wayland GUI probes; run with --ignored"]
fn q02_760_altr_self_modifier_e2e() {
    if !qt6_wayland_available() {
        eprintln!("SKIP q02_760: qt6 wayland platform plugin not installed");
        return;
    }
    let scratch = ScratchDir::new("q02_760");
    let sway = SwaySession::new(&scratch).expect("start headless sway");
    let mut inject = VkbdInjector::new(sway.socket(), &scratch).expect("start injector");
    let staged = stage::stage_qt(&scratch, 6).expect("stage qt6 plugin");

    // Phase 1: does this frontend/compositor pair deliver Key{AltR, ALT}?
    let xdg = kime::write_config(&scratch, DETECTOR_CONFIG).expect("write detector config");
    let detector = altr_round(&sway, &mut inject, &staged, &xdg, &scratch);
    if detector != "r가" {
        eprintln!(
            "SKIP q02_760: the AltR press did not carry its own modifier bit in this setup \
             (M-AltR-only binding did not fire; buffer {detector:?}) — the #760 shape does not \
             reproduce; engine unit tests (self_modifier.rs) cover the matching logic"
        );
        return;
    }

    // Phase 2: the stock plain-AltR binding must fire on the same shape.
    let xdg = kime::write_config(&scratch, kime::LATIN_CONFIG).expect("write kime config");
    let stock = altr_round(&sway, &mut inject, &staged, &xdg, &scratch);
    assert_eq!(
        stock, "r가",
        "plain AltR binding missed Key{{AltR, ALT}} (pre-fix #760: exact-match miss, buffer 'rrk')"
    );
}
