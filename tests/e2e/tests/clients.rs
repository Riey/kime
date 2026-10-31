//! Harness self-checks: every vendored C/C++ client must compile.
//!
//! Run: `cargo test -p kime-e2e --test clients -- --ignored`

use kime_e2e::cc;
use kime_e2e::paths::{self, Frontend};

/// Builds all on-demand clients (xim_client, xtest_key, vkbd_inject with
/// wayland-scanner glue, wlkbd_probe, qt_probe for Qt5 and Qt6). Catches
/// compiler/pkg-config problems before any GUI test needs the binaries.
#[test]
#[ignore = "e2e: needs cc/g++, pkg-config, wayland-scanner"]
fn build_all_clients() {
    cc::xim_client().expect("build xim_client");
    cc::xtest_key().expect("build xtest_key");
    cc::vkbd_inject().expect("build vkbd_inject");
    cc::wlkbd_probe().expect("build wlkbd_probe");
    cc::keymapless_kbd().expect("build keymapless_kbd");
    // The Qt5 probe only when this build has a qt5 plugin to test it against
    // — the same condition `qt::q5_smoke` skips on.
    if paths::immodule_opt(Frontend::Qt5).is_some() {
        cc::qt_probe(5).expect("build qt_probe5");
    }
    cc::qt_probe(6).expect("build qt_probe6");
    assert!(cc::gtk_probe_py().exists(), "gtk_probe.py missing");
}
