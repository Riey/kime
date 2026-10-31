//! XIM frontend end-to-end tests (kime-xim on a harness-owned Xvfb).
//!
//! Run: `cargo test -p kime-e2e --test xim -- --ignored --test-threads=1`

use std::time::Duration;

use kime_e2e::kime::{self, KimeXim};
use kime_e2e::paths::ScratchDir;
use kime_e2e::probes::{self, XimProbe};
use kime_e2e::proc;
use kime_e2e::x11::XvfbSession;

/// An X11 keycode with no `KeyCode::from_hardware_code` mapping (the table in
/// `src/engine/backend/src/keycode.rs` tops out at 134 after PR #752 added
/// SuperL/SuperR). xdotool cannot inject unmapped raw codes, hence xtest_key.
const POISON_KEYCODE: u32 = 247;

/// X11 keycodes for XF86AudioLowerVolume / XF86AudioRaiseVolume
/// (evdev 114/115 + 8). Pre-#769 the engine's hardware-code table carried the
/// raw evdev values 122/123 as Hangul/HangulHanja, so these X11-space codes
/// toggled the language / entered hanja mode instead of being bypassed.
const X11_VOLUME_DOWN: u32 = 122;
const X11_VOLUME_UP: u32 = 123;

/// X-SMOKE: full pipeline Xvfb → kime-xim → minimal C XIM client.
///
/// Types `gksrmf` + Enter (dubeolsik) and asserts the client received the
/// committed text `한글`. Validates the whole harness: display probing,
/// daemon spawn with local `libkime_engine.so`, XIM registration sync,
/// xdotool injection, and buffer watching.
#[test]
#[ignore = "e2e: spawns Xvfb and kime-xim; run with --ignored"]
fn x_smoke() {
    let scratch = ScratchDir::new("x_smoke");
    let x = XvfbSession::new(&scratch).expect("start Xvfb");
    let xdg = kime::write_config(&scratch, kime::HANGUL_CONFIG).expect("write kime config");
    let mut kime_xim = KimeXim::spawn(&x.display(), &xdg, &scratch).expect("start kime-xim");
    let probe = probes::spawn_xim_client(&x, &scratch).expect("start xim_client");

    x.type_text("gksrmf").expect("type gksrmf");
    x.key("Return").expect("press Return");

    probe
        .out
        .wait_contains("한글", Duration::from_secs(10))
        .expect("committed 한글");
    assert!(kime_xim.alive(), "kime-xim died during the test");
}

/// X-01 — #721/#731 (fixed by PR #722): kime-xim must survive a hardware
/// keycode with no `KeyCode::from_hardware_code` mapping.
///
/// Pre-fix, `handle_key_event` unwrapped the `Option` at
/// `src/frontends/xim/src/handler.rs:351`, so one unmapped key (Super in Java
/// apps / Hyprland xterm at report time) killed the XIM server for every
/// client. Post-fix it logs `Unknown hardware keycode: N`, returns the key to
/// the client unhandled, and keeps serving.
///
/// Three-part assertion: (1) the daemon is still alive 1s after the poison
/// key, (2) the log carries the unknown-keycode warning, (3) Hangul input in
/// the same client still commits afterwards.
#[test]
#[ignore = "e2e: spawns Xvfb and kime-xim; run with --ignored"]
fn x01_721_survives_unmapped_keycode() {
    let scratch = ScratchDir::new("x01_721_survives_unmapped_keycode");
    let x = XvfbSession::new(&scratch).expect("start Xvfb");
    let xdg = kime::write_config(&scratch, kime::HANGUL_CONFIG).expect("write kime config");
    let mut kime_xim = KimeXim::spawn(&x.display(), &xdg, &scratch).expect("start kime-xim");
    let probe = probes::spawn_xim_client(&x, &scratch).expect("start xim_client");

    // Baseline: the pipeline works before the poison key.
    x.type_text("gksrmf").expect("type gksrmf");
    x.key("Return").expect("press Return");
    probe
        .out
        .wait_contains("한글", Duration::from_secs(10))
        .expect("baseline committed 한글");

    // Poison key: raw unmapped X11 keycode via XTEST (press + release).
    x.raw_key_tap(POISON_KEYCODE)
        .expect("inject poison keycode");

    // (2) The daemon saw and logged the unmapped code...
    let needle = format!("Unknown hardware keycode: {POISON_KEYCODE}");
    proc::wait_for_line(&kime_xim.log, &[&needle], Duration::from_secs(10))
        .expect("kime-xim logged the unknown hardware keycode");
    // (1) ...and is still alive 1s later (pre-fix: dead at handler.rs:351).
    proc::sleep_ms(1000);
    assert!(
        kime_xim.alive(),
        "kime-xim died after an unmapped hardware keycode (#721); log tail:\n{}",
        proc::tail(&kime_xim.log)
    );

    // (3) Continued IM function: a second round still commits 한글.
    x.type_text("gksrmf").expect("type gksrmf again");
    x.key("Return").expect("press Return again");
    wait_hangul_count(&probe, 2).expect("second 한글 committed after the poison key");
    assert!(kime_xim.alive(), "kime-xim died during the second round");
}

/// X-02 — #603 (fixed by PR #769 fix 1): volume keys are bypassed to the app;
/// they must not toggle the language or open hanja mode.
///
/// kime-xim feeds X11-space keycodes (`xev.detail`) to the engine. Pre-fix
/// the table mapped 122/123 (raw evdev values for Hangul/Hanja, but X11
/// XF86AudioLowerVolume/RaiseVolume) to Hangul/HangulHanja, so VolumeDown
/// toggled to Latin mid-composition and VolumeUp spawned the candidate
/// window. Post-fix both are unmapped and bypassed.
///
/// Shape: preedit 한 (`gks`) → inject X11 122 and 123 via XTEST → assert no
/// language toggle (finishing `rmf` + Return still commits 한글, no literal
/// `rmf`), no kime-candidate-window child appeared, and kime-xim stayed alive.
#[test]
#[ignore = "e2e: spawns Xvfb and kime-xim; run with --ignored"]
fn x02_603_volume_key_bypassed() {
    let scratch = ScratchDir::new("x02_603_volume_key_bypassed");
    let x = XvfbSession::new(&scratch).expect("start Xvfb");
    let xdg = kime::write_config(&scratch, kime::HANGUL_CONFIG).expect("write kime config");
    let mut kime_xim = KimeXim::spawn(&x.display(), &xdg, &scratch).expect("start kime-xim");
    let probe = probes::spawn_xim_client(&x, &scratch).expect("start xim_client");

    // Compose 한 and leave it in preedit.
    x.type_text("gks").expect("type gks");

    // Volume keys mid-composition. Post-#769 both are unmapped in the
    // hardware-code table, so the daemon logs them and passes them through.
    x.raw_key_tap(X11_VOLUME_DOWN).expect("inject VolumeDown");
    x.raw_key_tap(X11_VOLUME_UP).expect("inject VolumeUp");
    let down = format!("Unknown hardware keycode: {X11_VOLUME_DOWN}");
    let up = format!("Unknown hardware keycode: {X11_VOLUME_UP}");
    proc::wait_for_line(&kime_xim.log, &[&down], Duration::from_secs(10))
        .expect("kime-xim bypassed VolumeDown as unmapped");
    proc::wait_for_line(&kime_xim.log, &[&up], Duration::from_secs(10))
        .expect("kime-xim bypassed VolumeUp as unmapped");

    // Give a (pre-fix) hanja-mode candidate window time to spawn, then assert
    // none did.
    proc::sleep_ms(500);
    let candidates = candidate_children(kime_xim.pid());
    assert!(
        candidates.is_empty(),
        "volume key spawned kime-candidate-window children {candidates:?} (#603 hanja regression)"
    );

    // No language toggle: the composition continues seamlessly into 한글.
    x.type_text("rmf").expect("type rmf");
    x.key("Return").expect("press Return");
    probe
        .out
        .wait_contains("한글", Duration::from_secs(10))
        .expect("committed 한글 after volume keys");
    let buffer = probe.out.read();
    assert!(
        !buffer.contains("rmf"),
        "literal latin leaked into the buffer ({buffer:?}) — VolumeDown toggled the language (#603)"
    );

    assert!(
        candidate_children(kime_xim.pid()).is_empty(),
        "kime-candidate-window children present at teardown (#603)"
    );
    assert!(
        kime_xim.alive(),
        "kime-xim died during the test; log tail:\n{}",
        proc::tail(&kime_xim.log)
    );
}

/// Wait until the XIM client's buffer holds `count` committed `한글`.
fn wait_hangul_count(probe: &XimProbe, count: usize) -> kime_e2e::Result<()> {
    proc::wait_until(
        &format!(
            "buffer {} to hold {count}x 한글 (last: {:?})",
            probe.out.path().display(),
            probe.out.read()
        ),
        Duration::from_secs(10),
        || probe.out.read().matches("한글").count() >= count,
    )
}

/// Pids of live `kime-candidate-window` children of `pid` (via
/// `/proc/<pid>/task/*/children`; comm is truncated to 15 bytes so match on
/// the `kime-candidate` prefix).
fn candidate_children(pid: i32) -> Vec<i32> {
    let mut out = Vec::new();
    let Ok(tasks) = std::fs::read_dir(format!("/proc/{pid}/task")) else {
        return out;
    };
    for task in tasks.flatten() {
        let Ok(children) = std::fs::read_to_string(task.path().join("children")) else {
            continue;
        };
        for child in children.split_whitespace() {
            let Ok(child_pid) = child.parse::<i32>() else {
                continue;
            };
            let comm =
                std::fs::read_to_string(format!("/proc/{child_pid}/comm")).unwrap_or_default();
            if comm.trim_end().starts_with("kime-candidate") {
                out.push(child_pid);
            }
        }
    }
    out
}

/// X-03 (#783, fix: #786): a GTK3 app on the XIM bridge must be able to
/// compose and commit — no preedit echo loop.
///
/// GTK3's XIM immodule negotiates `PREEDIT_CALLBACKS` (it draws the preedit
/// in its own widget) and reports the resulting cursor move back with
/// `SetICValues(XNSpotLocation)`. kime-xim answered every spot report by
/// tearing down and re-sending the preedit, which moved the cursor again — a
/// self-sustaining PreeditDone/PreeditStart/PreeditDraw echo at ~100k
/// messages/s in which nothing ever commits. FAILS on develop until #786
/// (skip the redraw for `PREEDIT_CALLBACKS` contexts) merges.
///
/// The GTK3 probe here uses `GTK_IM_MODULE=xim` (no kime immodule staging):
/// GTK3 → XIM → kime-xim, the exact #783 client path.
#[test]
#[ignore = "e2e: spawns Xvfb, kime-xim and a GTK3 app; run with --ignored"]
fn x03_783_gtk3_xim_client_commits() {
    let scratch = ScratchDir::new("x03_783");
    let x = XvfbSession::new(&scratch).expect("start Xvfb");
    let xdg = kime::write_config(&scratch, kime::HANGUL_CONFIG).expect("write kime config");
    let mut kime_xim = KimeXim::spawn(&x.display(), &xdg, &scratch).expect("start kime-xim");
    let probe = probes::spawn_gtk_probe_x11(
        &x,
        &[
            ("GTK_IM_MODULE".into(), "xim".into()),
            ("XMODIFIERS".into(), "@im=kime".into()),
        ],
        &probes::GtkProbeOpts {
            gtk_major: 3,
            textview: false,
        },
        &scratch,
    )
    .expect("start gtk3 probe over XIM");
    x.focus_window(probes::PROBE_TITLE).expect("focus probe");

    x.type_text("gksrmf").expect("type gksrmf");
    x.key("Return").expect("press Return");

    probe
        .buffer
        .wait_for("한글", Duration::from_secs(15))
        .expect(
            "GTK3-over-XIM never committed — preedit echo loop (#783; fails until #786 merges)",
        );
    assert!(kime_xim.alive(), "kime-xim died during the test");
}
