//! Environment hygiene: every spawned process starts from `env_clear()` plus a
//! small allowlist, so the harness can never leak the user's live session
//! (`WAYLAND_DISPLAY`, `DISPLAY`, `GTK_IM_MODULE=kime`, ...) into a test.

use std::ffi::OsStr;
use std::process::Command;

/// Variables inherited from the harness environment. Everything else —
/// including `DISPLAY`, `WAYLAND_DISPLAY`, `XDG_SESSION_*`, `*_IM_MODULE`,
/// `XMODIFIERS` — is dropped and must be set explicitly per test.
pub const ALLOWLIST: &[&str] = &["PATH", "HOME", "XDG_RUNTIME_DIR", "LANG"];

/// Names the environment asks the harness to forward on top of [`ALLOWLIST`],
/// read from `KIME_E2E_PASS_ENV` (comma separated).
///
/// An environment that keeps its GTK/Qt/GL runtime outside the paths those
/// libraries search by default — the nix devshell being the case in point —
/// sets this to hand probes their typelibs, GL drivers and fonts, so the
/// allowlist above stays free of distro-specific names. Session state
/// (`DISPLAY`, `WAYLAND_DISPLAY`, `*_IM_MODULE`) must never be listed.
pub fn passthrough_keys() -> Vec<String> {
    std::env::var("KIME_E2E_PASS_ENV")
        .unwrap_or_default()
        .split(',')
        .map(str::trim)
        .filter(|k| !k.is_empty())
        .map(str::to_string)
        .collect()
}

/// Value of a forwarded variable, or `None` when it is unset or not listed in
/// `KIME_E2E_PASS_ENV`.
pub fn passthrough_value(key: &str) -> Option<String> {
    passthrough_keys()
        .iter()
        .any(|k| k == key)
        .then(|| std::env::var(key).ok())
        .flatten()
}

/// `LD_LIBRARY_PATH` for every kime process and GUI probe: the local target
/// dir first (a system `libkime_engine.so` may be stale), then the forwarded
/// value, if any — the nix devshell points it at its wayland and GL libraries.
pub fn ld_library_path() -> String {
    let target = crate::paths::target_dir().display().to_string();
    match passthrough_value("LD_LIBRARY_PATH") {
        Some(rest) if !rest.is_empty() => format!("{target}:{rest}"),
        _ => target,
    }
}

/// A `Command` with a cleared environment plus [`ALLOWLIST`] and
/// [`passthrough_keys`]. Falls back to `LANG=C.UTF-8` when the host has no
/// `LANG` (XIM needs a UTF-8 locale).
pub fn clean_cmd(program: impl AsRef<OsStr>) -> Command {
    let mut cmd = Command::new(program);
    cmd.env_clear();
    for key in ALLOWLIST
        .iter()
        .map(|k| k.to_string())
        .chain(passthrough_keys())
    {
        if let Ok(val) = std::env::var(&key) {
            cmd.env(key, val);
        }
    }
    if std::env::var_os("LANG").is_none() {
        cmd.env("LANG", "C.UTF-8");
    }
    cmd
}

/// `XDG_RUNTIME_DIR`, which wayland sockets require. Errors (instead of
/// silently producing confusing connect failures) when unset.
pub fn xdg_runtime_dir() -> crate::Result<String> {
    std::env::var("XDG_RUNTIME_DIR")
        .map_err(|_| "XDG_RUNTIME_DIR is not set; wayland tests need it".to_string())
}
