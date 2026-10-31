//! Wayland frontend end-to-end tests (kime-wayland on a harness-owned
//! headless sway, `zwp_input_method_v2` path).
//!
//! Run: `cargo test -p kime-e2e --test wayland -- --ignored --test-threads=1`
//!
//! Every test spawns its own sway session, virtual-keyboard injector
//! (BEFORE kime-wayland — see [`kime_e2e::inject`]) and probe client; teardown
//! is by exact-pid RAII guards.

use std::path::Path;
use std::time::Duration;

use kime_e2e::inject::{key, VkbdInjector};
use kime_e2e::kime::{self, KimeWayland};
use kime_e2e::paths::ScratchDir;
use kime_e2e::probes::{self, GtkProbeOpts, GuiProbe, ImEvent};
use kime_e2e::proc;
use kime_e2e::sway::SwaySession;
use kime_e2e::wldebug;

/// Full v2 stack for text-input tests. Field order gives the teardown order
/// probe → kime-wayland → injector → sway → scratch (structs drop fields in
/// declaration order).
struct WlStack {
    probe: GuiProbe,
    kime: KimeWayland,
    inject: VkbdInjector,
    _sway: SwaySession,
    _scratch: ScratchDir,
}

impl WlStack {
    /// sway → injector → kime-wayland (per-test config) → GTK4 Entry probe
    /// (text-input-v3), then wait until the compositor activated kime for the
    /// probe (keys injected earlier would be bypassed and reach the client raw).
    fn new(name: &str, config: &str) -> WlStack {
        let scratch = ScratchDir::new(name);
        let sway = SwaySession::new(&scratch).expect("start sway");
        let inject = VkbdInjector::new(sway.socket(), &scratch).expect("start injector");
        let xdg = kime::write_config(&scratch, config).expect("write kime config");
        let mut kime =
            KimeWayland::spawn(sway.socket(), &xdg, &scratch).expect("start kime-wayland");
        let probe = probes::spawn_gtk_probe_wayland(
            sway.socket(),
            &GtkProbeOpts {
                gtk_major: 4,
                textview: false,
            },
            &scratch,
        )
        .expect("start gtk4 wayland probe");
        kime.wait_activated(Duration::from_secs(10))
            .expect("input method activated for the probe");
        WlStack {
            probe,
            kime,
            inject,
            _sway: sway,
            _scratch: scratch,
        }
    }
}

/// Preedit strings logged by the probe so far, in order.
fn preedits(preedit_log: &Path) -> Vec<String> {
    probes::read_im_log(preedit_log)
        .into_iter()
        .filter_map(|ev| match ev {
            ImEvent::Preedit(s) => Some(s),
            ImEvent::Commit(_) => None,
        })
        .collect()
}

/// Wait until the most recent preedit event equals `expected`.
fn wait_last_preedit(probe: &GuiProbe, expected: &str, timeout: Duration) -> kime_e2e::Result<()> {
    proc::wait_until(
        &format!(
            "last preedit event in {} == {expected:?} (events: {:?})",
            probe.preedit_log.display(),
            preedits(&probe.preedit_log)
        ),
        timeout,
        || preedits(&probe.preedit_log).last().map(String::as_str) == Some(expected),
    )
}

/// True if `needle` appears in `hay` in order (not necessarily contiguous).
fn is_subsequence(needle: &[&str], hay: &[String]) -> bool {
    let mut it = hay.iter();
    needle.iter().all(|n| it.any(|h| h == n))
}

/// Assert the probe buffer is (stably) empty. The probe's dump writes are
/// atomic (temp file + rename), but a commit still in flight could land just
/// after a single read; sampling across several dump periods turns "empty
/// right now" into "stably empty".
fn assert_buffer_empty(probe: &GuiProbe, ctx: &str) {
    for _ in 0..5 {
        let b = probe.buffer.read();
        assert_eq!(b, "", "{ctx} (buffer: {b:?})");
        proc::sleep_ms(120);
    }
}

/// Pids of `kime-candidate-window` children of `pid` (any thread).
fn candidate_children(pid: i32) -> Vec<i32> {
    let mut found = Vec::new();
    let Ok(tasks) = std::fs::read_dir(format!("/proc/{pid}/task")) else {
        return found;
    };
    for task in tasks.flatten() {
        let Ok(children) = std::fs::read_to_string(task.path().join("children")) else {
            continue;
        };
        for tok in children.split_whitespace() {
            let Ok(child) = tok.parse::<i32>() else {
                continue;
            };
            let comm = std::fs::read_to_string(format!("/proc/{child}/comm")).unwrap_or_default();
            if comm.contains("kime-candidate") {
                found.push(child);
            }
        }
    }
    found
}

/// W-SMOKE: full pipeline sway → injector → kime-wayland → GTK4 probe
/// (text-input-v3).
///
/// Types `gksrmf` + Enter (dubeolsik) and asserts the probe committed `한글`
/// with the full preedit sequence ㅎ→하→한→ㄱ→그→글. Validates the whole
/// wayland harness: socket parsing, injector-before-kime ordering, keymap
/// forwarding, grab routing and activation sync.
#[test]
#[ignore = "e2e: spawns sway and kime-wayland; run with --ignored"]
fn w_smoke() {
    let mut s = WlStack::new("w_smoke", kime::HANGUL_CONFIG);

    s.inject.tap_seq(&key::GKSRMF).expect("inject gksrmf");
    s.inject.tap(key::ENTER).expect("inject Enter");

    s.probe
        .buffer
        .wait_contains("한글", Duration::from_secs(10))
        .expect("committed 한글");
    let seen = preedits(&s.probe.preedit_log);
    assert!(
        is_subsequence(&["ㅎ", "하", "한", "ㄱ", "그", "글"], &seen),
        "preedit sequence ㅎ하한ㄱ그글 not observed; got {seen:?}"
    );
    assert!(s.kime.alive(), "kime-wayland died during the test");
}

/// W-01 (#714, #654, #668, #528 — PR #772 Fix A): a key that changes neither
/// preedit nor commit state must not produce a `zwp_input_method_v2.commit`
/// request.
///
/// Under the double-buffered input-method-v2 → text-input-v3 semantics a bare
/// `commit` applies an "empty preedit, no commit string" state and yields a
/// spurious `done` at the client, which Firefox/Chromium interpret as an empty
/// composition replacing the current selection (Mozilla bug 1977196). Guarded
/// by `state_changed` in `process_input_result_v2`
/// (`src/frontends/wayland/src/state.rs`).
///
/// Protocol half: after typed text (Latin) lone Ctrl/Shift/Alt/Super presses
/// and an arrow key must add zero new `commit` requests to the
/// `WAYLAND_DEBUG` trace. Behavioral half: a selection survives Ctrl+C and a
/// lone Shift.
///
/// The Hangul phase asserts the strict contract with a live preedit too: a
/// bypassed modifier adds zero commit requests and zero preedit events. The
/// engine reports `HAS_PREEDIT` for bypassed keys, so this half FAILS on
/// develop until #781 (fix: #788, `last_preedit` tracking) merges — pre-#788
/// every lone modifier re-sent the identical preedit plus a commit.
#[test]
#[ignore = "e2e: spawns sway and kime-wayland; run with --ignored"]
fn w01_714_no_spurious_commit() {
    // --- Latin category, committed text ---
    {
        let mut s = WlStack::new("w01_714_latin", kime::LATIN_CONFIG);
        s.inject.tap_seq(&[30, 48]).expect("inject ab"); // a, b
        s.probe
            .buffer
            .wait_for("ab", Duration::from_secs(10))
            .expect("buffer ab");

        // Protocol assertion: lone modifiers + arrow ⇒ zero new commit requests.
        let m = wldebug::marker(&s.kime.trace);
        for code in [key::LEFTCTRL, key::LEFTSHIFT, key::LEFTALT, key::LEFTMETA] {
            s.inject.press(code).expect("press modifier");
            s.inject.release(code).expect("release modifier");
        }
        s.inject.tap(key::LEFT).expect("tap Left");
        proc::sleep_ms(700);
        let commits =
            wldebug::count_requests_after(&s.kime.trace, m, "zwp_input_method_v2", "commit");
        assert_eq!(
            commits, 0,
            "lone modifiers/arrow produced {commits} spurious zwp_input_method_v2.commit \
             request(s) (#714)"
        );

        // Behavioral assertion: Shift+Left selection, Ctrl+C, lone Shift —
        // the selection must not be replaced by an empty composition.
        let m2 = wldebug::marker(&s.kime.trace);
        s.inject.press(key::LEFTSHIFT).expect("press Shift");
        s.inject.tap(key::LEFT).expect("tap Left");
        s.inject.release(key::LEFTSHIFT).expect("release Shift");
        s.inject.press(key::LEFTCTRL).expect("press Ctrl");
        s.inject.tap(46).expect("tap c"); // Ctrl+C
        s.inject.release(key::LEFTCTRL).expect("release Ctrl");
        s.inject.press(key::LEFTSHIFT).expect("press lone Shift");
        s.inject
            .release(key::LEFTSHIFT)
            .expect("release lone Shift");
        proc::sleep_ms(700);
        // wait_for (not a one-shot read): tolerates the dump-period lag of the
        // probe's atomic buffer writes. The pre-fix failure is a *permanent*
        // "a", which still times out here.
        s.probe
            .buffer
            .wait_for("ab", Duration::from_secs(10))
            .expect("selection was clobbered after Ctrl+C / lone Shift (#714)");
        let commits =
            wldebug::count_requests_after(&s.kime.trace, m2, "zwp_input_method_v2", "commit");
        assert_eq!(
            commits, 0,
            "selection/copy/lone-Shift produced {commits} spurious commit request(s) (#714)"
        );
        assert!(s.kime.alive(), "kime-wayland died during the Latin phase");
    }

    // --- Hangul category, live preedit ---
    {
        let mut s = WlStack::new("w01_714_hangul", kime::HANGUL_CONFIG);
        s.inject.tap_seq(&key::DKS).expect("inject dks"); // preedit 안
        wait_last_preedit(&s.probe, "안", Duration::from_secs(10)).expect("preedit 안");

        let m = wldebug::marker(&s.kime.trace);
        let n_preedit = preedits(&s.probe.preedit_log).len();
        for code in [key::LEFTCTRL, key::LEFTSHIFT, key::LEFTALT, key::LEFTMETA] {
            s.inject.press(code).expect("press lone modifier");
            s.inject.release(code).expect("release lone modifier");
        }
        proc::sleep_ms(700);

        let commits =
            wldebug::count_requests_after(&s.kime.trace, m, "zwp_input_method_v2", "commit");
        assert_eq!(
            commits, 0,
            "lone modifiers over a live preedit produced {commits} redundant commit \
             request(s) — an unchanged preedit was re-sent (#714 residual, #781; fails \
             until #788 merges)"
        );
        let after = preedits(&s.probe.preedit_log);
        assert!(
            after[n_preedit..].is_empty(),
            "lone modifiers produced duplicate preedit event(s) at the client: {:?} \
             (#781; fails until #788 merges)",
            &after[n_preedit..]
        );
        assert_buffer_empty(
            &s.probe,
            "lone modifiers committed text out of a live preedit (#714)",
        );
        assert!(s.kime.alive(), "kime-wayland died during the Hangul phase");
    }
}

/// W-02 (#666 — PR #772 Fix B): a held key keeps repeating across the
/// preedit/committed-text boundary.
///
/// While Backspace repeat ticks are consumed by the engine they delete preedit
/// jamo; once the preedit empties the ticks become bypassed and kime must keep
/// repeating the key toward the client itself (`repeat_bypassed` in
/// `handle_timer_ev`, `src/frontends/wayland/src/state.rs`) — the compositor
/// applies no second repeat delay and, per the 3.2.0/#743 regression, the
/// repeat must not stop entirely at the boundary (which would leave `한`).
///
/// Repeat cadence comes from the compositor (`repeat_info` on the grab; kime's
/// config has no repeat knobs), so hold durations use generous margins over
/// sway's defaults (600 ms delay, 25 Hz).
#[test]
#[ignore = "e2e: spawns sway and kime-wayland; run with --ignored"]
fn w02_666_repeat_crosses_boundary() {
    // --- Part A: Hangul, repeat crosses from preedit into committed text ---
    {
        let mut s = WlStack::new("w02_666_hangul", kime::HANGUL_CONFIG);
        s.inject.tap_seq(&key::GKSRMF).expect("inject gksrmf");
        s.probe
            .buffer
            .wait_for("한", Duration::from_secs(10))
            .expect("committed 한");
        wait_last_preedit(&s.probe, "글", Duration::from_secs(10)).expect("preedit 글");

        s.inject.press(key::BACKSPACE).expect("hold Backspace");
        proc::sleep_ms(2500);
        s.inject.release(key::BACKSPACE).expect("release Backspace");

        s.probe
            .buffer
            .wait_for("", Duration::from_secs(10))
            .expect("held Backspace deleted preedit AND committed text (#666)");
        assert_buffer_empty(&s.probe, "held Backspace left committed text behind (#666)");
        wait_last_preedit(&s.probe, "", Duration::from_secs(5))
            .expect("preedit cleared by held Backspace");
        assert!(s.kime.alive(), "kime-wayland died during Part A");
    }

    // --- Part B: Latin, fully bypassed held Backspace keeps repeating ---
    {
        let mut s = WlStack::new("w02_666_latin", kime::LATIN_CONFIG);
        s.inject.tap_seq(&[30, 48, 46]).expect("inject abc"); // a, b, c
        s.probe
            .buffer
            .wait_for("abc", Duration::from_secs(10))
            .expect("buffer abc");

        s.inject.press(key::BACKSPACE).expect("hold Backspace");
        proc::sleep_ms(2000);
        s.inject.release(key::BACKSPACE).expect("release Backspace");

        s.probe
            .buffer
            .wait_for("", Duration::from_secs(10))
            .expect("bypassed held Backspace kept repeating (#666)");
        assert_buffer_empty(&s.probe, "held Backspace stopped repeating early (#666)");
        assert!(s.kime.alive(), "kime-wayland died during Part B");
    }
}

/// W-03 (#744 — PR #745): with the keyboard grab held but no text input
/// enabled, every key press AND release must be forwarded to the focused
/// client.
///
/// A plain wl_keyboard client (no text-input) receives 5 injected taps of
/// KEY_A as exactly 5 press + 5 release events. Pre-fix the
/// `is_pressed && !grab_activate` branch in `src/frontends/wayland/src/state.rs`
/// dropped them all, leaving apps deaf to keyboard input outside text fields.
#[test]
#[ignore = "e2e: spawns sway and kime-wayland; run with --ignored"]
fn w03_744_bypass_without_text_input() {
    let scratch = ScratchDir::new("w03_744");
    let sway = SwaySession::new(&scratch).expect("start sway");
    let mut inject = VkbdInjector::new(sway.socket(), &scratch).expect("start injector");
    let xdg = kime::write_config(&scratch, kime::HANGUL_CONFIG).expect("write kime config");
    let mut kime_wl =
        KimeWayland::spawn(sway.socket(), &xdg, &scratch).expect("start kime-wayland");
    let probe = probes::spawn_wlkbd_probe(sway.socket(), &scratch).expect("start wlkbd probe");
    proc::wait_for_line(&probe.log, &["enter"], Duration::from_secs(10))
        .expect("probe keyboard focus (enter event)");

    for _ in 0..5 {
        inject.tap(30).expect("tap a"); // KEY_A
    }

    proc::wait_until(
        &format!("10 key events at the client (got {:?})", probe.key_events()),
        Duration::from_secs(10),
        || probe.key_events().len() >= 10,
    )
    .expect("keys forwarded without an enabled text input (#744)");
    proc::sleep_ms(500); // catch any surplus events
    let events = probe.key_events();
    let presses = events.iter().filter(|&&(p, c)| p && c == 30).count();
    let releases = events.iter().filter(|&&(p, c)| !p && c == 30).count();
    assert_eq!(
        (presses, releases, events.len()),
        (5, 5, 10),
        "expected exactly 5 press + 5 release of code 30, got {events:?} (#744)"
    );
    assert!(kime_wl.alive(), "kime-wayland died during the test");
}

/// W-04 (#603 — PR #769 fix 1): volume keys are bypassed to the application —
/// no language toggle, no hanja candidate window.
///
/// Pre-fix `from_hardware_code` mapped raw evdev 121/122/123 as
/// Hangul/HangulHanja while frontends pass X11-space codes (evdev+8), so
/// XF86AudioLowerVolume (evdev 114 → X11 122) toggled Hangul/Latin. This test
/// also pins the wayland frontend's `+8` conversion in
/// `src/frontends/wayland/src/state.rs`: with preedit 한 live, KEY_VOLUMEDOWN
/// must spawn no kime-candidate-window child and typing must continue in
/// Hangul (final buffer `한글`, not literal `rmf`).
#[test]
#[ignore = "e2e: spawns sway and kime-wayland; run with --ignored"]
fn w04_603_volume_key_bypassed() {
    let mut s = WlStack::new("w04_603", kime::HANGUL_CONFIG);

    s.inject.tap_seq(&key::GKSRMF[..3]).expect("inject gks"); // preedit 한
    wait_last_preedit(&s.probe, "한", Duration::from_secs(10)).expect("preedit 한");

    s.inject.tap(key::VOLUMEDOWN).expect("tap VolumeDown");
    proc::sleep_ms(700);

    let zombies = candidate_children(s.kime.pid());
    assert!(
        zombies.is_empty(),
        "VolumeDown spawned kime-candidate-window (pids {zombies:?}) (#603)"
    );
    assert_buffer_empty(
        &s.probe,
        "VolumeDown committed text out of the preedit (#603)",
    );

    // Language must not have toggled: finish the syllables and commit.
    s.inject.tap_seq(&key::GKSRMF[3..]).expect("inject rmf");
    s.inject.tap(key::ENTER).expect("inject Enter");
    s.probe
        .buffer
        .wait_for("한글", Duration::from_secs(10))
        .expect("Hangul typing continued after VolumeDown (#603)");
    assert!(s.kime.alive(), "kime-wayland died during the test");
}

/// W-05 (#782, fix: #789): kime-wayland must survive a seat keyboard that has
/// no keymap.
///
/// The precondition is a keyboard whose keymap is NULL — a virtual keyboard
/// created without a keymap upload (a seat with no keyboard at all sends no
/// grab keymap event and never triggers the bug). wlroots then reports
/// `keyboard_grab.keymap(format=NO_KEYMAP, size=0)`; kime forwarded it
/// verbatim to its own `zwp_virtual_keyboard_v1`, the compositor rejected the
/// zero-size mmap with a fatal protocol error, and kime aborted (panic at the
/// roundtrip unwrap in `main.rs`). This is what breaks naive headless/CI
/// setups — the other tests in this file dodge it via injector-before-kime
/// ordering. FAILS on develop until #789 merges.
#[test]
#[ignore = "e2e: spawns sway and kime-wayland; run with --ignored"]
fn w05_782_survives_keymapless_seat() {
    let scratch = ScratchDir::new("w05_782");
    let sway = SwaySession::new(&scratch).expect("start sway");
    let _kbd = probes::spawn_keymapless_kbd(sway.socket(), &scratch)
        .expect("start keymapless keyboard holder");
    let xdg = kime::write_config(&scratch, kime::HANGUL_CONFIG).expect("write kime config");
    let mut kime_wl =
        KimeWayland::spawn_unready(sway.socket(), &xdg, &scratch).expect("spawn kime-wayland");
    // Pre-#789 kime forwards the empty keymap and aborts within ~1s of the
    // grab; 3s of liveness proves the guard.
    proc::sleep_ms(3000);
    assert!(
        kime_wl.alive(),
        "kime-wayland died on a keymapless seat keyboard — the compositor's \
         format=0/size=0 grab keymap was forwarded verbatim (#782; fails until #789 \
         merges); trace: {}",
        kime_wl.trace.display()
    );
}
