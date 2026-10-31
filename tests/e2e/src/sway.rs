//! Headless sway compositor per test.
//!
//! `WLR_BACKENDS=headless WLR_LIBINPUT_NO_DEVICES=1 WLR_RENDERER=pixman
//! sway -V -c <minimal conf>`; `-V` is mandatory — without it sway never logs
//! the `Running compositor on wayland display 'wayland-N'` line the harness
//! parses for the socket name (sockets are auto-assigned, so concurrent
//! sessions don't collide).

use std::process::Stdio;
use std::time::Duration;

use crate::envs::{clean_cmd, xdg_runtime_dir};
use crate::paths::ScratchDir;
use crate::proc::Proc;
use crate::Result;

const CONF: &str = "output HEADLESS-1 resolution 1280x720\nxwayland disable\n";

pub struct SwaySession {
    socket: String,
    /// sway's verbose log (stderr+stdout).
    pub log: std::path::PathBuf,
    _proc: Proc,
}

impl SwaySession {
    pub fn new(scratch: &ScratchDir) -> Result<SwaySession> {
        xdg_runtime_dir()?; // fail early with a clear message
        let conf = scratch.file("sway.conf");
        std::fs::write(&conf, CONF).map_err(|e| format!("cannot write {}: {e}", conf.display()))?;
        let log = scratch.file("sway.log");
        let logfile = std::fs::File::create(&log)
            .map_err(|e| format!("cannot create {}: {e}", log.display()))?;
        let logfile2 = logfile
            .try_clone()
            .map_err(|e| format!("cannot dup sway log fd: {e}"))?;
        let mut cmd = clean_cmd("sway");
        cmd.env("WLR_BACKENDS", "headless")
            .env("WLR_LIBINPUT_NO_DEVICES", "1")
            .env("WLR_RENDERER", "pixman")
            .arg("-V")
            .arg("-c")
            .arg(&conf)
            .stdout(Stdio::from(logfile))
            .stderr(Stdio::from(logfile2));
        let mut proc = Proc::spawn(&mut cmd, "sway")?;
        let line = proc.wait_ready_line(
            &log,
            &["Running compositor on wayland display"],
            Duration::from_secs(10),
        )?;
        // ... on wayland display 'wayland-1'
        let socket = line
            .split('\'')
            .nth(1)
            .ok_or_else(|| format!("cannot parse sway socket from line: {line}"))?
            .to_string();
        Ok(SwaySession {
            socket,
            log,
            _proc: proc,
        })
    }

    /// Socket name for `WAYLAND_DISPLAY`, e.g. `"wayland-1"`.
    pub fn socket(&self) -> &str {
        &self.socket
    }
}
