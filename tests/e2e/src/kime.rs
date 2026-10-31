//! Spawning the kime daemons under test, with per-test config.
//!
//! Both daemons run with the local target dir first on `LD_LIBRARY_PATH`
//! (the system `libkime_engine.so` is stale on dev machines) and a per-test
//! `XDG_CONFIG_HOME` written by [`write_config`].

use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;

use crate::envs::{self, clean_cmd};
use crate::paths::{self, ScratchDir};
use crate::proc::{self, Proc};
use crate::Result;

/// Minimal config: start in Hangul mode (dubeolsik default layout).
pub const HANGUL_CONFIG: &str = "engine:\n  default_category: Hangul\n";
/// Minimal config: start in Latin mode.
pub const LATIN_CONFIG: &str = "engine:\n  default_category: Latin\n";

/// Write `yaml` to `<scratch>/xdg-config/kime/config.yaml`; returns the
/// directory to use as `XDG_CONFIG_HOME`.
pub fn write_config(scratch: &ScratchDir, yaml: &str) -> Result<PathBuf> {
    let xdg = scratch.subdir("xdg-config");
    let kime_dir = xdg.join("kime");
    std::fs::create_dir_all(&kime_dir)
        .map_err(|e| format!("cannot create {}: {e}", kime_dir.display()))?;
    let cfg = kime_dir.join("config.yaml");
    std::fs::write(&cfg, yaml).map_err(|e| format!("cannot write {}: {e}", cfg.display()))?;
    Ok(xdg)
}

/// `kime-wayland` under `WAYLAND_DEBUG=1`, stderr (protocol trace + logs)
/// captured to [`KimeWayland::trace`] for [`crate::wldebug`] assertions.
pub struct KimeWayland {
    /// stderr trace file (WAYLAND_DEBUG protocol lines + kime logs).
    pub trace: PathBuf,
    proc: Proc,
}

impl KimeWayland {
    /// Spawn against `socket` and wait until kime has forwarded a keymap to
    /// its own `zwp_virtual_keyboard_v1` — which only happens once the
    /// injector's keyboard exists, so construct [`crate::inject::VkbdInjector`]
    /// first.
    pub fn spawn(
        socket: &str,
        xdg_config_home: &Path,
        scratch: &ScratchDir,
    ) -> Result<KimeWayland> {
        let bin = paths::bin("kime-wayland");
        let trace = scratch.file("kime-wayland.trace");
        let tracefile = std::fs::File::create(&trace)
            .map_err(|e| format!("cannot create {}: {e}", trace.display()))?;
        let mut cmd = clean_cmd(&bin);
        cmd.env("WAYLAND_DISPLAY", socket)
            .env("WAYLAND_DEBUG", "1")
            .env("LD_LIBRARY_PATH", envs::ld_library_path())
            .env("XDG_CONFIG_HOME", xdg_config_home)
            .env("RUST_BACKTRACE", "1")
            .args(["--log", "debug"])
            .stdout(Stdio::null())
            .stderr(tracefile);
        let mut kw = Self::spawn_unready(socket, xdg_config_home, scratch)?;
        // Startup sequence: bind input_method_v2 → grab keyboard → receive the
        // (injector-provided) keymap → forward it to kime's virtual keyboard.
        let trace = kw.trace.clone();
        kw.proc.wait_ready_line(
            &trace,
            &["zwp_virtual_keyboard_v1", ".keymap("],
            Duration::from_secs(10),
        )?;
        Ok(kw)
    }

    /// Spawn WITHOUT waiting for a keymap forward — for tests that
    /// deliberately give kime a seat with no keymapped keyboard (w05, #782):
    /// there is nothing to forward, and the pre-#789 binary dies outright.
    pub fn spawn_unready(
        socket: &str,
        xdg_config_home: &Path,
        scratch: &ScratchDir,
    ) -> Result<KimeWayland> {
        let bin = paths::bin("kime-wayland");
        let trace = scratch.file("kime-wayland.trace");
        let tracefile = std::fs::File::create(&trace)
            .map_err(|e| format!("cannot create {}: {e}", trace.display()))?;
        let mut cmd = clean_cmd(&bin);
        cmd.env("WAYLAND_DISPLAY", socket)
            .env("WAYLAND_DEBUG", "1")
            .env("LD_LIBRARY_PATH", envs::ld_library_path())
            .env("XDG_CONFIG_HOME", xdg_config_home)
            .env("RUST_BACKTRACE", "1")
            .args(["--log", "debug"])
            .stdout(Stdio::null())
            .stderr(tracefile);
        let proc = Proc::spawn(&mut cmd, "kime-wayland")?;
        Ok(KimeWayland { trace, proc })
    }

    pub fn pid(&self) -> i32 {
        self.proc.pid()
    }

    pub fn alive(&mut self) -> bool {
        self.proc.alive()
    }

    /// Wait until the compositor activated kime's input method for a focused
    /// text-input client (`.activate()` followed by `.done(`). Call this
    /// AFTER spawning a text-input probe and BEFORE injecting keys — keys
    /// injected earlier are bypassed and reach the client raw.
    pub fn wait_activated(&mut self, timeout: Duration) -> Result<()> {
        let trace = self.trace.clone();
        let name = self.proc.name().to_string();
        let ready = proc::wait_until(
            &format!("input method activation in {}", trace.display()),
            timeout,
            || {
                if !self.proc.alive() {
                    return true; // break out; reported below
                }
                let Ok(bytes) = std::fs::read(&trace) else {
                    return false;
                };
                let text = String::from_utf8_lossy(&bytes);
                match text.rfind(".activate()") {
                    Some(idx) => text[idx..].contains(".done("),
                    None => false,
                }
            },
        );
        if let Some(status) = self.proc.exit_status() {
            return Err(format!(
                "{name} exited ({status}) while waiting for activation; trace tail:\n{}",
                proc::tail(&trace)
            ));
        }
        ready
    }

    /// Wait for a trace line containing all `needles` (protocol events, kime
    /// log lines, ...), erroring early if kime-wayland exits.
    pub fn wait_trace(&mut self, needles: &[&str], timeout: Duration) -> Result<String> {
        let trace = self.trace.clone();
        self.proc.wait_ready_line(&trace, needles, timeout)
    }
}

/// `kime-xim --log debug` on a harness-owned X display.
pub struct KimeXim {
    /// stderr log file (`--log debug`), used e.g. for the
    /// `Unknown hardware keycode` assertion of X-01 (#721).
    pub log: PathBuf,
    proc: Proc,
}

impl KimeXim {
    /// Spawn and wait until the XIM server registered `@server=kime` on the
    /// root window (polled via `xprop`; skipped if xprop is unavailable — the
    /// XIM client spawner retries on XOpenIM failure either way).
    pub fn spawn(display: &str, xdg_config_home: &Path, scratch: &ScratchDir) -> Result<KimeXim> {
        let bin = paths::bin("kime-xim");
        let log = scratch.file("kime-xim.log");
        let logfile = std::fs::File::create(&log)
            .map_err(|e| format!("cannot create {}: {e}", log.display()))?;
        let mut cmd = clean_cmd(&bin);
        cmd.env("DISPLAY", display)
            .env("LD_LIBRARY_PATH", envs::ld_library_path())
            .env("XDG_CONFIG_HOME", xdg_config_home)
            .env("RUST_BACKTRACE", "1")
            .args(["--log", "debug"])
            .stdout(Stdio::null())
            .stderr(logfile);
        let mut proc = Proc::spawn(&mut cmd, "kime-xim")?;
        // Deterministic readiness: XIM servers register in the XIM_SERVERS
        // root property. Poll it when xprop exists.
        let display = display.to_string();
        let has_xprop = which("xprop");
        if has_xprop {
            let ready = proc::wait_until(
                &format!("XIM_SERVERS to list kime on {display}"),
                Duration::from_secs(10),
                || {
                    if !proc.alive() {
                        return true; // break out; checked below
                    }
                    xim_server_registered(&display)
                },
            );
            if let Some(status) = proc.exit_status() {
                return Err(format!(
                    "kime-xim exited at startup ({status}); log tail:\n{}",
                    proc::tail(&log)
                ));
            }
            ready?;
        }
        Ok(KimeXim { log, proc })
    }

    pub fn pid(&self) -> i32 {
        self.proc.pid()
    }

    pub fn alive(&mut self) -> bool {
        self.proc.alive()
    }
}

fn which(bin: &str) -> bool {
    std::env::var_os("PATH")
        .map(|paths| std::env::split_paths(&paths).any(|d| d.join(bin).is_file()))
        .unwrap_or(false)
}

fn xim_server_registered(display: &str) -> bool {
    clean_cmd("xprop")
        .env("DISPLAY", display)
        .args(["-root", "XIM_SERVERS"])
        .output()
        .map(|o| {
            o.status.success() && {
                let s = String::from_utf8_lossy(&o.stdout);
                s.contains("XIM_SERVERS") && !s.contains("no such atom")
            }
        })
        .unwrap_or(false)
}
