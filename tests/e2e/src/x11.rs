//! Fresh `Xvfb` per test with free-display probing, plus `xdotool` (XTEST)
//! wrappers and a raw-keycode injector for keys xdotool cannot synthesize.

use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;

use crate::envs::clean_cmd;
use crate::paths::ScratchDir;
use crate::proc::{self, Proc};
use crate::{cc, Result};

/// A headless X server owned by the test. Never reuses the session's `:0`.
pub struct XvfbSession {
    display_num: u32,
    _proc: Proc,
}

impl XvfbSession {
    /// Probe for a free display number (`:50`..) and start `Xvfb` on it,
    /// retrying on collision with concurrently starting servers.
    pub fn new(scratch: &ScratchDir) -> Result<XvfbSession> {
        let seed = std::process::id();
        let mut last_err = String::new();
        for attempt in 0..30u32 {
            let n = 50 + (seed.wrapping_mul(7).wrapping_add(attempt * 13)) % 150;
            if socket_path(n).exists() || lock_path(n).exists() {
                continue;
            }
            let log = scratch.file(&format!("xvfb-{n}.log"));
            let logfile = std::fs::File::create(&log)
                .map_err(|e| format!("cannot create {}: {e}", log.display()))?;
            let mut cmd = clean_cmd("Xvfb");
            cmd.arg(format!(":{n}"))
                .args(["-screen", "0", "1024x768x24"])
                .stdout(Stdio::null())
                .stderr(logfile);
            let mut proc = Proc::spawn(&mut cmd, &format!("Xvfb :{n}"))?;
            // Wait for the socket; if Xvfb dies first (display race), try the next N.
            let up = proc::wait_until(&format!("Xvfb :{n} socket"), Duration::from_secs(5), || {
                socket_path(n).exists() || !proc.alive()
            });
            match up {
                Ok(()) if proc.alive() && socket_path(n).exists() => {
                    return Ok(XvfbSession {
                        display_num: n,
                        _proc: proc,
                    });
                }
                _ => {
                    last_err = format!(
                        "Xvfb :{n} did not come up (collision?): {}",
                        proc::tail(&log)
                    );
                }
            }
        }
        Err(format!(
            "no free X display found after 30 attempts; last error: {last_err}"
        ))
    }

    /// Display string, e.g. `":97"`.
    pub fn display(&self) -> String {
        format!(":{}", self.display_num)
    }

    /// Run an `xdotool` subcommand against this display; returns stdout.
    pub fn xdotool(&self, args: &[&str]) -> Result<String> {
        let output = clean_cmd("xdotool")
            .env("DISPLAY", self.display())
            .args(args)
            .output()
            .map_err(|e| format!("failed to run xdotool: {e}"))?;
        if !output.status.success() {
            return Err(format!(
                "xdotool {args:?} on {} failed ({}): {}",
                self.display(),
                output.status,
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    /// Type text via XTEST with a 100ms inter-key delay (verified pacing).
    pub fn type_text(&self, text: &str) -> Result<()> {
        self.xdotool(&["type", "--delay", "100", "--", text])?;
        Ok(())
    }

    /// Press a named key (X keysym name, e.g. `Return`, `Control_R`).
    pub fn key(&self, key: &str) -> Result<()> {
        self.xdotool(&["key", "--", key])?;
        Ok(())
    }

    /// Wait for a window whose name matches `title_pattern`, focus it with
    /// `windowfocus --sync`, and return the window id.
    pub fn focus_window(&self, title_pattern: &str) -> Result<String> {
        let mut wid = String::new();
        proc::wait_until(
            &format!("window matching {title_pattern:?} on {}", self.display()),
            Duration::from_secs(10),
            || match self.xdotool(&["search", "--name", title_pattern]) {
                Ok(out) if !out.is_empty() => {
                    wid = out.lines().next().unwrap_or_default().to_string();
                    true
                }
                _ => false,
            },
        )?;
        self.xdotool(&["windowfocus", "--sync", &wid])?;
        Ok(wid)
    }

    /// Inject a raw X11 keycode (press+release) via XTEST — for keycodes
    /// xdotool cannot map (e.g. the unmapped poison key of #721).
    pub fn raw_key_tap(&self, keycode: u32) -> Result<()> {
        self.raw_key(keycode, "tap")
    }

    /// Inject a raw X11 keycode transition: `action` is `press`/`release`/`tap`.
    pub fn raw_key(&self, keycode: u32, action: &str) -> Result<()> {
        let bin: PathBuf = cc::xtest_key()?;
        let output = clean_cmd(&bin)
            .env("DISPLAY", self.display())
            .arg(keycode.to_string())
            .arg(action)
            .output()
            .map_err(|e| format!("failed to run xtest_key: {e}"))?;
        if !output.status.success() {
            return Err(format!(
                "xtest_key {keycode} {action} failed ({}): {}",
                output.status,
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        Ok(())
    }
}

fn socket_path(n: u32) -> PathBuf {
    PathBuf::from(format!("/tmp/.X11-unix/X{n}"))
}

fn lock_path(n: u32) -> PathBuf {
    PathBuf::from(format!("/tmp/.X{n}-lock"))
}
