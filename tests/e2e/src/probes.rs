//! Probe applications (the "apps under test") and their output watchers.
//!
//! Every probe dumps its committed text to a file the harness polls
//! ([`BufferWatcher`]) and appends IM events to `<out>.preedit` as
//! `p:<preedit>` / `c:<commit>` lines ([`read_im_log`]).

use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;

use crate::envs::{self, clean_cmd};
use crate::paths::ScratchDir;
use crate::proc::{self, Proc};
use crate::x11::XvfbSession;
use crate::{cc, Result};

/// Window title shared by all GUI probes (for `xdotool search --name`).
pub const PROBE_TITLE: &str = "kimeprobe";

/// Polls a probe's committed-text dump file.
pub struct BufferWatcher {
    path: PathBuf,
}

impl BufferWatcher {
    pub fn new(path: PathBuf) -> BufferWatcher {
        BufferWatcher { path }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Current buffer content ("" while the file doesn't exist yet).
    pub fn read(&self) -> String {
        std::fs::read(&self.path)
            .map(|b| String::from_utf8_lossy(&b).into_owned())
            .unwrap_or_default()
    }

    /// Wait until the buffer equals `expected` exactly.
    pub fn wait_for(&self, expected: &str, timeout: Duration) -> Result<()> {
        proc::wait_until(
            &format!(
                "buffer {} == {expected:?} (last: {:?})",
                self.path.display(),
                self.read()
            ),
            timeout,
            || self.read() == expected,
        )
    }

    /// Wait until the buffer contains `needle`.
    pub fn wait_contains(&self, needle: &str, timeout: Duration) -> Result<()> {
        proc::wait_until(
            &format!(
                "buffer {} to contain {needle:?} (last: {:?})",
                self.path.display(),
                self.read()
            ),
            timeout,
            || self.read().contains(needle),
        )
    }
}

/// One IM event from a probe's `.preedit` log.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImEvent {
    /// `p:<text>` — preedit changed (empty string = preedit cleared).
    Preedit(String),
    /// `c:<text>` — text committed (Qt probe only; GTK commits land in the buffer).
    Commit(String),
}

/// Parse a probe's `.preedit` log (missing file = no events yet).
pub fn read_im_log(path: &Path) -> Vec<ImEvent> {
    let Ok(bytes) = std::fs::read(path) else {
        return Vec::new();
    };
    String::from_utf8_lossy(&bytes)
        .lines()
        .filter_map(|l| {
            l.strip_prefix("p:")
                .map(|s| ImEvent::Preedit(s.to_string()))
                .or_else(|| l.strip_prefix("c:").map(|s| ImEvent::Commit(s.to_string())))
        })
        .collect()
}

/// A spawned GUI probe (GTK or Qt): buffer watcher + IM-event/cursor files.
pub struct GuiProbe {
    pub buffer: BufferWatcher,
    /// `.preedit` log path (parse with [`read_im_log`]).
    pub preedit_log: PathBuf,
    /// `.cursor` dump path (GTK probe only; decimal char offset).
    pub cursor_file: PathBuf,
    /// Probe stderr log.
    pub log: PathBuf,
    proc: Proc,
}

impl GuiProbe {
    pub fn pid(&self) -> i32 {
        self.proc.pid()
    }

    pub fn alive(&mut self) -> bool {
        self.proc.alive()
    }

    /// Current cursor offset (GTK probe), if dumped yet.
    pub fn cursor(&self) -> Option<usize> {
        std::fs::read_to_string(&self.cursor_file)
            .ok()
            .and_then(|s| s.trim().parse().ok())
    }
}

/// Options for the GTK probe.
pub struct GtkProbeOpts {
    /// 3 or 4.
    pub gtk_major: u8,
    /// Use a `Gtk.TextView` (accepts Enter/Tab) instead of a `Gtk.Entry`.
    pub textview: bool,
}

/// GTK probe on a harness X display, with staged-immodule env from
/// [`crate::stage`]. Waits for the probe's `READY` line; callers still need
/// [`XvfbSession::focus_window`]`(`[`PROBE_TITLE`]`)` before typing.
pub fn spawn_gtk_probe_x11(
    x: &XvfbSession,
    extra_env: &[(String, String)],
    opts: &GtkProbeOpts,
    scratch: &ScratchDir,
) -> Result<GuiProbe> {
    let mut cmd = clean_cmd("python3");
    cmd.env("DISPLAY", x.display())
        .env("GDK_BACKEND", "x11")
        .env("LD_LIBRARY_PATH", envs::ld_library_path());
    spawn_gtk_common(cmd, extra_env, opts, scratch)
}

/// GTK probe on a headless sway socket (text-input-v3 — no immodule env, so
/// GTK talks to kime-wayland via the compositor).
pub fn spawn_gtk_probe_wayland(
    socket: &str,
    opts: &GtkProbeOpts,
    scratch: &ScratchDir,
) -> Result<GuiProbe> {
    let mut cmd = clean_cmd("python3");
    cmd.env("WAYLAND_DISPLAY", socket)
        .env("GDK_BACKEND", "wayland");
    spawn_gtk_common(cmd, &[], opts, scratch)
}

fn spawn_gtk_common(
    mut cmd: std::process::Command,
    extra_env: &[(String, String)],
    opts: &GtkProbeOpts,
    scratch: &ScratchDir,
) -> Result<GuiProbe> {
    let out = scratch.file(&format!("gtk{}-buffer.txt", opts.gtk_major));
    let log = scratch.file(&format!("gtk{}-probe.log", opts.gtk_major));
    let logfile =
        std::fs::File::create(&log).map_err(|e| format!("cannot create {}: {e}", log.display()))?;
    for (k, v) in extra_env {
        cmd.env(k, v);
    }
    cmd.arg(cc::gtk_probe_py())
        .arg(opts.gtk_major.to_string())
        .arg(&out);
    if opts.textview {
        cmd.arg("--textview");
    }
    cmd.stdout(Stdio::null()).stderr(logfile);
    let name = format!("gtk{}-probe", opts.gtk_major);
    let mut proc = Proc::spawn(&mut cmd, &name)?;
    proc.wait_ready_line(&log, &["READY"], Duration::from_secs(15))?;
    Ok(GuiProbe {
        preedit_log: out.with_extension("txt.preedit"),
        cursor_file: out.with_extension("txt.cursor"),
        buffer: BufferWatcher::new(out),
        log,
        proc,
    })
}

/// A spawned `gtk_reset_probe.py` client: a direct `Gtk.IMMulticontext` (no
/// Entry/TextView) that logs every "commit" signal with its handler nesting
/// depth — the instrument for the #562 re-entrant-reset tests.
pub struct GtkResetProbe {
    /// `.commits` log: one `d<depth>:<text>` line per "commit" signal
    /// (read with [`GtkResetProbe::commit_lines`]; `d2:`+ = re-entrant).
    pub commits_log: PathBuf,
    /// `.preedit` log path (parse with [`read_im_log`]).
    pub preedit_log: PathBuf,
    /// `.events` trace (key forwarding + reset markers).
    pub events_log: PathBuf,
    /// Probe stderr log.
    pub log: PathBuf,
    proc: Proc,
}

impl GtkResetProbe {
    pub fn pid(&self) -> i32 {
        self.proc.pid()
    }

    pub fn alive(&mut self) -> bool {
        self.proc.alive()
    }

    /// All `d<depth>:<text>` lines committed so far (missing file = none yet).
    pub fn commit_lines(&self) -> Vec<String> {
        let Ok(bytes) = std::fs::read(&self.commits_log) else {
            return Vec::new();
        };
        String::from_utf8_lossy(&bytes)
            .lines()
            .map(str::to_string)
            .collect()
    }
}

/// GTK3 re-entrant-reset probe on a harness X display with staged-immodule
/// env. `reset_in_commit` passes `--reset-in-commit` (the client calls
/// `im.reset()` from its commit handler — the kime#562 Firefox pattern).
/// Waits for `READY` and verifies the IMMulticontext picked the `kime` slave;
/// callers still need [`XvfbSession::focus_window`]`(`[`PROBE_TITLE`]`)`.
pub fn spawn_gtk_reset_probe_x11(
    x: &XvfbSession,
    extra_env: &[(String, String)],
    reset_in_commit: bool,
    scratch: &ScratchDir,
) -> Result<GtkResetProbe> {
    let out = scratch.file("gtk3-reset");
    let log = scratch.file("gtk3-reset-probe.log");
    let logfile =
        std::fs::File::create(&log).map_err(|e| format!("cannot create {}: {e}", log.display()))?;
    let mut cmd = clean_cmd("python3");
    cmd.env("DISPLAY", x.display())
        .env("GDK_BACKEND", "x11")
        .env("LD_LIBRARY_PATH", envs::ld_library_path());
    for (k, v) in extra_env {
        cmd.env(k, v);
    }
    cmd.arg(cc::gtk_reset_probe_py()).arg(&out);
    if reset_in_commit {
        cmd.arg("--reset-in-commit");
    }
    cmd.stdout(Stdio::null()).stderr(logfile);
    let mut proc = Proc::spawn(&mut cmd, "gtk3-reset-probe")?;
    proc.wait_ready_line(&log, &["READY"], Duration::from_secs(15))?;
    // The probe prints CONTEXT_ID before READY; anything but "kime" means GTK
    // silently fell back to another slave and the test would be vacuous.
    proc::wait_for_line(&log, &["CONTEXT_ID:kime"], Duration::from_secs(1))
        .map_err(|e| format!("IMMulticontext slave is not kime: {e}"))?;
    Ok(GtkResetProbe {
        commits_log: out.with_extension("commits"),
        preedit_log: out.with_extension("preedit"),
        events_log: out.with_extension("events"),
        log,
        proc,
    })
}

/// Qt probe (QLineEdit) on a harness X display with staged plugin env.
pub fn spawn_qt_probe_x11(
    x: &XvfbSession,
    qt_major: u32,
    extra_env: &[(String, String)],
    scratch: &ScratchDir,
) -> Result<GuiProbe> {
    let mut cmd = clean_cmd(cc::qt_probe(qt_major)?);
    cmd.env("DISPLAY", x.display())
        .env("QT_QPA_PLATFORM", "xcb");
    spawn_qt_common(cmd, qt_major, extra_env, scratch)
}

/// Qt probe on a headless sway socket (`QT_QPA_PLATFORM=wayland`; needs the
/// qtN-wayland platform plugin — used by Q-02/#760).
pub fn spawn_qt_probe_wayland(
    socket: &str,
    qt_major: u32,
    extra_env: &[(String, String)],
    scratch: &ScratchDir,
) -> Result<GuiProbe> {
    let mut cmd = clean_cmd(cc::qt_probe(qt_major)?);
    cmd.env("WAYLAND_DISPLAY", socket)
        .env("QT_QPA_PLATFORM", "wayland");
    spawn_qt_common(cmd, qt_major, extra_env, scratch)
}

fn spawn_qt_common(
    mut cmd: std::process::Command,
    qt_major: u32,
    extra_env: &[(String, String)],
    scratch: &ScratchDir,
) -> Result<GuiProbe> {
    let out = scratch.file(&format!("qt{qt_major}-buffer.txt"));
    let log = scratch.file(&format!("qt{qt_major}-probe.log"));
    let logfile =
        std::fs::File::create(&log).map_err(|e| format!("cannot create {}: {e}", log.display()))?;
    cmd.env("LD_LIBRARY_PATH", envs::ld_library_path());
    for (k, v) in extra_env {
        cmd.env(k, v);
    }
    cmd.arg(&out).stdout(Stdio::null()).stderr(logfile);
    let name = format!("qt{qt_major}-probe");
    let mut proc = Proc::spawn(&mut cmd, &name)?;
    proc.wait_ready_line(&log, &["READY"], Duration::from_secs(15))?;
    Ok(GuiProbe {
        preedit_log: out.with_extension("txt.preedit"),
        cursor_file: out.with_extension("txt.cursor"),
        buffer: BufferWatcher::new(out),
        log,
        proc,
    })
}

/// The minimal C XIM client (PreeditNothing — GTK3 as an XIM client echo-loops
/// against kime-xim, so this is the mandated substitute). Focuses its own
/// window on map; committed text is appended to [`XimProbe::out`].
pub struct XimProbe {
    pub out: BufferWatcher,
    /// Client stderr log.
    pub log: PathBuf,
    proc: Proc,
}

impl XimProbe {
    pub fn pid(&self) -> i32 {
        self.proc.pid()
    }

    pub fn alive(&mut self) -> bool {
        self.proc.alive()
    }
}

/// Spawn the XIM client with `XMODIFIERS=@im=kime`, retrying while the XIM
/// server is still registering (XOpenIM has no retry of its own).
pub fn spawn_xim_client(x: &XvfbSession, scratch: &ScratchDir) -> Result<XimProbe> {
    let bin = cc::xim_client()?;
    let out = scratch.file("xim-buffer.txt");
    let log = scratch.file("xim-client.log");
    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    loop {
        let logfile = std::fs::File::create(&log)
            .map_err(|e| format!("cannot create {}: {e}", log.display()))?;
        let mut cmd = clean_cmd(&bin);
        cmd.env("DISPLAY", x.display())
            .env("XMODIFIERS", "@im=kime")
            .arg(&out)
            .stdout(Stdio::null())
            .stderr(logfile);
        let mut proc = Proc::spawn(&mut cmd, "xim_client")?;
        match proc.wait_ready_line(&log, &["READY"], Duration::from_secs(10)) {
            Ok(_) => {
                return Ok(XimProbe {
                    out: BufferWatcher::new(out),
                    log,
                    proc,
                })
            }
            Err(e) => {
                // XOpenIM failed because kime-xim isn't registered yet → retry.
                if std::time::Instant::now() > deadline {
                    return Err(format!("xim_client never became ready: {e}"));
                }
                proc::sleep_ms(200);
            }
        }
    }
}

/// A seat keyboard with NO keymap: the client creates a
/// `zwp_virtual_keyboard_v1` and never uploads a keymap, so wlroots reports
/// `keymap(format=NO_KEYMAP, size=0)` to an input-method grab — the #782
/// precondition. Keep the struct alive while the scenario runs (dropping it
/// removes the keyboard).
pub struct KeymaplessKbd {
    proc: Proc,
}

impl KeymaplessKbd {
    pub fn pid(&self) -> i32 {
        self.proc.pid()
    }
}

/// Spawn the keymapless-keyboard holder on `socket` and wait for `READY`.
pub fn spawn_keymapless_kbd(socket: &str, scratch: &ScratchDir) -> Result<KeymaplessKbd> {
    let bin = cc::keymapless_kbd()?;
    let log = scratch.file("keymapless-kbd.log");
    let logfile =
        std::fs::File::create(&log).map_err(|e| format!("cannot create {}: {e}", log.display()))?;
    let errfile = logfile
        .try_clone()
        .map_err(|e| format!("cannot clone log handle: {e}"))?;
    let mut cmd = clean_cmd(&bin);
    cmd.env("WAYLAND_DISPLAY", socket)
        .stdin(Stdio::null())
        .stdout(logfile)
        .stderr(errfile);
    let mut proc = Proc::spawn(&mut cmd, "keymapless_kbd")?;
    proc.wait_ready_line(&log, &["READY"], Duration::from_secs(10))?;
    Ok(KeymaplessKbd { proc })
}

/// Plain wl_keyboard client (no text input) for W-03/#744; events appear in
/// [`WlKbdProbe::log`] as `<ts_ms> press|release <code>` lines.
pub struct WlKbdProbe {
    /// Event log file written by the probe.
    pub log: PathBuf,
    /// Probe stderr.
    pub stderr_log: PathBuf,
    proc: Proc,
}

impl WlKbdProbe {
    pub fn pid(&self) -> i32 {
        self.proc.pid()
    }

    pub fn alive(&mut self) -> bool {
        self.proc.alive()
    }

    /// Parsed `(press, code)` key events seen so far.
    pub fn key_events(&self) -> Vec<(bool, u32)> {
        let Ok(bytes) = std::fs::read(&self.log) else {
            return Vec::new();
        };
        String::from_utf8_lossy(&bytes)
            .lines()
            .filter_map(|l| {
                let mut parts = l.split_whitespace();
                let _ts = parts.next()?;
                let kind = parts.next()?;
                let code = parts.next()?.parse().ok()?;
                match kind {
                    "press" => Some((true, code)),
                    "release" => Some((false, code)),
                    _ => None,
                }
            })
            .collect()
    }
}

/// Spawn the wl_keyboard probe on `socket`; waits for `READY` (surface mapped).
/// Callers should then wait for an `enter` line in [`WlKbdProbe::log`] before
/// injecting.
pub fn spawn_wlkbd_probe(socket: &str, scratch: &ScratchDir) -> Result<WlKbdProbe> {
    let bin = cc::wlkbd_probe()?;
    let out = scratch.file("wlkbd-events.txt");
    let log = scratch.file("wlkbd-probe.log");
    let logfile =
        std::fs::File::create(&log).map_err(|e| format!("cannot create {}: {e}", log.display()))?;
    let mut cmd = clean_cmd(&bin);
    cmd.env("WAYLAND_DISPLAY", socket)
        .arg(&out)
        .stdout(Stdio::null())
        .stderr(logfile);
    let mut proc = Proc::spawn(&mut cmd, "wlkbd_probe")?;
    proc.wait_ready_line(&log, &["READY"], Duration::from_secs(10))?;
    Ok(WlKbdProbe {
        log: out,
        stderr_log: log,
        proc,
    })
}
