//! GTK4 immodule end-to-end tests (staged local `libkime-gtk4.so` on Xvfb).
//!
//! Run: `cargo test -p kime-e2e --test gtk4 -- --ignored --test-threads=1`

use std::path::Path;
use std::time::Duration;

use kime_e2e::kime;
use kime_e2e::paths::ScratchDir;
use kime_e2e::probes::{self, GtkProbeOpts, GuiProbe, ImEvent, PROBE_TITLE};
use kime_e2e::proc;
use kime_e2e::stage::{self, Staged};
use kime_e2e::x11::XvfbSession;

const WAIT: Duration = Duration::from_secs(10);

/// Stage the local GTK4 immodule, write a Hangul config, and spawn the probe
/// on `x`. The caller must still `focus_window(PROBE_TITLE)` before typing.
fn spawn_probe(x: &XvfbSession, scratch: &ScratchDir, textview: bool) -> (GuiProbe, Staged) {
    let staged = stage::stage_gtk4(scratch).expect("stage gtk4 immodule");
    let xdg = kime::write_config(scratch, kime::HANGUL_CONFIG).expect("write kime config");
    let mut env = staged.env.clone();
    env.push(("XDG_CONFIG_HOME".into(), xdg.display().to_string()));
    let opts = GtkProbeOpts {
        gtk_major: 4,
        textview,
    };
    let probe = probes::spawn_gtk_probe_x11(x, &env, &opts, scratch).expect("start gtk4 probe");
    (probe, staged)
}

/// Number of IM events logged so far (marker for [`wait_new_preedit`]).
fn im_len(log: &Path) -> usize {
    probes::read_im_log(log).len()
}

/// Wait until at least one new IM event arrived after `since` and the latest
/// event is `Preedit(expected)`.
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

/// G4-SMOKE: Xvfb → staged local `libkime-gtk4.so` → GTK4 Entry probe.
///
/// Types `gksrmf` + Enter (dubeolsik) and asserts the committed text `한글`
/// with the full preedit stream `ㅎ하한ㄱ그글`, plus a `/proc/<pid>/maps`
/// check that the LOCAL staged module loaded (a system-wide copy exists on
/// dev machines and would silently shadow it otherwise).
#[test]
#[ignore = "e2e: spawns Xvfb and a GTK4 app; run with --ignored"]
fn g4_smoke() {
    let scratch = ScratchDir::new("g4_smoke");
    let x = XvfbSession::new(&scratch).expect("start Xvfb");
    let (probe, staged) = spawn_probe(&x, &scratch, false);
    x.focus_window(PROBE_TITLE).expect("focus probe window");

    x.type_text("gksrmf").expect("type gksrmf");
    x.key("Return").expect("press Return");

    probe.buffer.wait_for("한글", WAIT).expect("committed 한글");
    stage::maps_check_staged(probe.pid(), &staged).expect("local gtk4 immodule loaded");
    assert_preedit_sequence(&probe.preedit_log, &["ㅎ", "하", "한", "ㄱ", "그", "글"]);
}

/// G4-01 (#606/#613, PR #775 fix 1): a single Enter press while a preedit is
/// visible commits the syllable AND inserts the newline.
///
/// Pre-fix, the GTK3-style deferral swallowed bypassed keys on GTK4: the
/// first Enter only committed `안` and a second press was needed for the
/// newline (`gtk_im_context_filter_key` never reached the widget while the
/// outer `TRUE` marked the event handled). The second round guards against
/// off-by-one accumulation across repeated Enter presses.
#[test]
#[ignore = "e2e: spawns Xvfb and a GTK4 app; run with --ignored"]
fn g4_01_606_enter_commits_and_newlines() {
    let scratch = ScratchDir::new("g4_01_606");
    let x = XvfbSession::new(&scratch).expect("start Xvfb");
    let (probe, staged) = spawn_probe(&x, &scratch, true);
    x.focus_window(PROBE_TITLE).expect("focus probe window");

    // Round 1: dks → preedit 안, then ONE Return.
    let since = im_len(&probe.preedit_log);
    x.type_text("dks").expect("type dks");
    wait_new_preedit(&probe.preedit_log, "안", since);
    stage::maps_check_staged(probe.pid(), &staged).expect("local gtk4 immodule loaded");
    x.key("Return").expect("press Return");
    probe
        .buffer
        .wait_for("안\n", WAIT)
        .expect("single Enter must commit 안 and insert the newline (#606)");

    // Round 2: no accumulation — dks, Return, again.
    let since = im_len(&probe.preedit_log);
    x.type_text("dks").expect("type dks");
    wait_new_preedit(&probe.preedit_log, "안", since);
    x.key("Return").expect("press Return");
    probe
        .buffer
        .wait_for("안\n안\n", WAIT)
        .expect("second dks+Enter round must yield 안\\n안\\n");
}

/// G4-02 (#606/#613, PR #775 fix 1): bypassed Tab and arrow keys reach the
/// widget on first press while a preedit is visible.
///
/// Tab: `dks` + Tab → `안\t`. Arrow: with the caret at the end, `dks` + Left
/// commits `안` and then moves the caret left of it (cursor offset dump);
/// typing `d` + Return afterwards inserts `ㅇ` at the MOVED caret, proving
/// the arrow reached the widget before the next preedit started.
#[test]
#[ignore = "e2e: spawns Xvfb and a GTK4 app; run with --ignored"]
fn g4_02_606_tab_and_arrow_reach_widget() {
    let scratch = ScratchDir::new("g4_02_606");
    let x = XvfbSession::new(&scratch).expect("start Xvfb");
    let (probe, staged) = spawn_probe(&x, &scratch, true);
    x.focus_window(PROBE_TITLE).expect("focus probe window");

    // Tab: dks → preedit 안, one Tab → committed 안 + tab.
    let since = im_len(&probe.preedit_log);
    x.type_text("dks").expect("type dks");
    wait_new_preedit(&probe.preedit_log, "안", since);
    stage::maps_check_staged(probe.pid(), &staged).expect("local gtk4 immodule loaded");
    x.key("Tab").expect("press Tab");
    probe
        .buffer
        .wait_for("안\t", WAIT)
        .expect("single Tab must commit 안 and insert the tab (#613)");

    // Arrow: dks → preedit 안, Left → commit lands, caret moves before it.
    let since = im_len(&probe.preedit_log);
    x.type_text("dks").expect("type dks");
    wait_new_preedit(&probe.preedit_log, "안", since);
    x.key("Left").expect("press Left");
    probe
        .buffer
        .wait_for("안\t안", WAIT)
        .expect("Left must first commit the preedit 안");
    proc::wait_until(
        &format!(
            "cursor to move to offset 2 after Left (last: {:?})",
            probe.cursor()
        ),
        WAIT,
        || probe.cursor() == Some(2),
    )
    .expect("Left must reach the widget and move the caret (#613)");

    // Typing at the moved caret: d → preedit ㅇ, Return commits it in place.
    let since = im_len(&probe.preedit_log);
    x.type_text("d").expect("type d");
    wait_new_preedit(&probe.preedit_log, "ㅇ", since);
    x.key("Return").expect("press Return");
    probe
        .buffer
        .wait_for("안\tㅇ\n안", WAIT)
        .expect("ㅇ must be inserted at the caret position Left moved to");
}
