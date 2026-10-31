//! Virtual-keyboard key injection into a headless sway session.
//!
//! ORDERING IS CRITICAL: construct the [`VkbdInjector`] BEFORE
//! [`crate::kime::KimeWayland`]. The injector uploads a real xkb keymap and
//! thereby gives the seat a keyboard; without it, sway hands kime an empty
//! `format=0/size=0` keymap on grab and kime-wayland aborts.

use std::io::Write;
use std::os::unix::ffi::OsStrExt;
use std::process::Stdio;
use std::time::Duration;

use crate::envs::clean_cmd;
use crate::paths::ScratchDir;
use crate::proc::Proc;
use crate::{cc, Result};

/// Common evdev key codes for tests (X11 keycode = evdev + 8).
pub mod key {
    pub const BACKSPACE: u32 = 14;
    pub const ENTER: u32 = 28;
    pub const TAB: u32 = 15;
    pub const ESC: u32 = 1;
    pub const LEFTCTRL: u32 = 29;
    pub const LEFTSHIFT: u32 = 42;
    pub const LEFTALT: u32 = 56;
    pub const LEFTMETA: u32 = 125;
    pub const RIGHTCTRL: u32 = 97;
    pub const RIGHTALT: u32 = 100;
    pub const LEFT: u32 = 105;
    pub const VOLUMEDOWN: u32 = 114;

    /// `gksrmf` → 한글 (dubeolsik).
    pub const GKSRMF: [u32; 6] = [34, 37, 31, 19, 50, 33];
    /// `dks` → preedit 안 (dubeolsik).
    pub const DKS: [u32; 3] = [32, 37, 31];
}

/// Persistent `zwp_virtual_keyboard_v1` injector process, driven through a
/// fifo the harness holds open read-write (so the injector never sees EOF).
pub struct VkbdInjector {
    fifo: std::fs::File,
    _proc: Proc,
}

impl VkbdInjector {
    /// Compile (if needed) and start the injector against `socket`, waiting
    /// for its `READY` line (keymap uploaded and roundtripped).
    pub fn new(socket: &str, scratch: &ScratchDir) -> Result<VkbdInjector> {
        let bin = cc::vkbd_inject()?;
        let fifo_path = scratch.file("inject.fifo");
        let c_path = std::ffi::CString::new(fifo_path.as_os_str().as_bytes())
            .map_err(|e| format!("bad fifo path: {e}"))?;
        if unsafe { libc::mkfifo(c_path.as_ptr(), 0o600) } != 0 {
            return Err(format!(
                "mkfifo {} failed: {}",
                fifo_path.display(),
                std::io::Error::last_os_error()
            ));
        }
        // O_RDWR on a fifo never blocks and keeps it open across writes
        // (the `exec 3<>fifo` trick).
        let fifo = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(&fifo_path)
            .map_err(|e| format!("cannot open fifo {}: {e}", fifo_path.display()))?;
        let stdin = fifo
            .try_clone()
            .map_err(|e| format!("cannot dup fifo fd: {e}"))?;
        let log = scratch.file("inject.log");
        let logfile = std::fs::File::create(&log)
            .map_err(|e| format!("cannot create {}: {e}", log.display()))?;
        let mut cmd = clean_cmd(&bin);
        cmd.env("WAYLAND_DISPLAY", socket)
            .stdin(Stdio::from(stdin))
            .stdout(Stdio::null())
            .stderr(logfile);
        let mut proc = Proc::spawn(&mut cmd, "vkbd_inject")?;
        proc.wait_ready_line(&log, &["READY"], Duration::from_secs(5))?;
        Ok(VkbdInjector { fifo, _proc: proc })
    }

    /// Press + release (25ms apart, injector-side).
    pub fn tap(&mut self, code: u32) -> Result<()> {
        self.send(format!("k {code}"))
    }

    /// Press and hold.
    pub fn press(&mut self, code: u32) -> Result<()> {
        self.send(format!("p {code}"))
    }

    /// Release a held key.
    pub fn release(&mut self, code: u32) -> Result<()> {
        self.send(format!("r {code}"))
    }

    /// Tap a sequence of keys in order.
    pub fn tap_seq(&mut self, codes: &[u32]) -> Result<()> {
        for &code in codes {
            self.tap(code)?;
        }
        Ok(())
    }

    fn send(&mut self, line: String) -> Result<()> {
        writeln!(self.fifo, "{line}").map_err(|e| format!("injector fifo write failed: {e}"))?;
        self.fifo
            .flush()
            .map_err(|e| format!("injector fifo flush failed: {e}"))?;
        // The injector roundtrips per command; this pacing only keeps bursts
        // human-ish and gives the compositor time to route each key.
        crate::proc::sleep_ms(40);
        Ok(())
    }
}
