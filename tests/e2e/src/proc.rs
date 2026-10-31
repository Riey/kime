//! RAII process guard and polling/synchronization helpers.
//!
//! Never kill by name — the user may run real `kime-xim`/`kime-wayland`
//! daemons. [`Proc`] kills only the exact pid it spawned.

use std::path::Path;
use std::process::{Child, Command, ExitStatus};
use std::time::{Duration, Instant};

use crate::Result;

pub const POLL: Duration = Duration::from_millis(50);

/// RAII guard around a spawned child: on drop, SIGTERM the exact pid, wait up
/// to 2s, then SIGKILL, and always reap.
pub struct Proc {
    child: Child,
    name: String,
}

impl Proc {
    /// Spawn `cmd`, labeling errors and the guard with `name`.
    pub fn spawn(cmd: &mut Command, name: &str) -> Result<Proc> {
        let child = cmd
            .spawn()
            .map_err(|e| format!("failed to spawn {name} ({:?}): {e}", cmd.get_program()))?;
        Ok(Proc {
            child,
            name: name.to_string(),
        })
    }

    pub fn pid(&self) -> i32 {
        self.child.id() as i32
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    /// True while the child has not exited (accurate: uses waitpid, not kill-0).
    pub fn alive(&mut self) -> bool {
        matches!(self.child.try_wait(), Ok(None))
    }

    /// Exit status if the child already exited.
    pub fn exit_status(&mut self) -> Option<ExitStatus> {
        self.child.try_wait().ok().flatten()
    }

    /// Wait for `pat` (all substrings) in `log`, erroring early if the child
    /// exits first. Use for readiness lines like `READY` or socket announcements.
    pub fn wait_ready_line(
        &mut self,
        log: &Path,
        needles: &[&str],
        timeout: Duration,
    ) -> Result<String> {
        let start = Instant::now();
        loop {
            if let Some(line) = find_line(log, needles) {
                return Ok(line);
            }
            if let Some(status) = self.exit_status() {
                return Err(format!(
                    "{} exited ({status}) before {needles:?} appeared in {}\n--- log tail ---\n{}",
                    self.name,
                    log.display(),
                    tail(log)
                ));
            }
            if start.elapsed() > timeout {
                return Err(format!(
                    "timed out ({timeout:?}) waiting for {needles:?} from {} in {}\n--- log tail ---\n{}",
                    self.name,
                    log.display(),
                    tail(log)
                ));
            }
            std::thread::sleep(POLL);
        }
    }
}

impl Drop for Proc {
    fn drop(&mut self) {
        if !self.alive() {
            return; // already exited and reaped by try_wait
        }
        let pid = self.pid();
        unsafe { libc::kill(pid, libc::SIGTERM) };
        let start = Instant::now();
        while start.elapsed() < Duration::from_secs(2) {
            if !self.alive() {
                return;
            }
            std::thread::sleep(POLL);
        }
        eprintln!("[e2e] {} (pid {pid}) ignored SIGTERM; killing", self.name);
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// True if a process with this pid exists (kill-0; zombies count as alive).
pub fn pid_alive(pid: i32) -> bool {
    unsafe { libc::kill(pid, 0) == 0 }
}

/// Poll `cond` until true or `timeout`; `what` names the condition in errors.
pub fn wait_until(what: &str, timeout: Duration, mut cond: impl FnMut() -> bool) -> Result<()> {
    let start = Instant::now();
    loop {
        if cond() {
            return Ok(());
        }
        if start.elapsed() > timeout {
            return Err(format!("timed out ({timeout:?}) waiting for {what}"));
        }
        std::thread::sleep(POLL);
    }
}

/// Wait until a line containing all `needles` appears in `log` (which may not
/// exist yet). Returns the first matching line.
pub fn wait_for_line(log: &Path, needles: &[&str], timeout: Duration) -> Result<String> {
    let start = Instant::now();
    loop {
        if let Some(line) = find_line(log, needles) {
            return Ok(line);
        }
        if start.elapsed() > timeout {
            return Err(format!(
                "timed out ({timeout:?}) waiting for {needles:?} in {}\n--- log tail ---\n{}",
                log.display(),
                tail(log)
            ));
        }
        std::thread::sleep(POLL);
    }
}

fn find_line(log: &Path, needles: &[&str]) -> Option<String> {
    let content = std::fs::read(log).ok()?;
    let content = String::from_utf8_lossy(&content);
    content
        .lines()
        .find(|l| needles.iter().all(|n| l.contains(n)))
        .map(str::to_string)
}

/// Last ~2KiB of a log file, for error messages.
pub fn tail(log: &Path) -> String {
    match std::fs::read(log) {
        Ok(b) => {
            let s = String::from_utf8_lossy(&b);
            let mut start = s.len().saturating_sub(2048);
            while !s.is_char_boundary(start) {
                start += 1;
            }
            s[start..].to_string()
        }
        Err(e) => format!("<unreadable: {e}>"),
    }
}

/// Plain sleep for pacing (readiness waits should use the wait_* helpers).
pub fn sleep_ms(ms: u64) {
    std::thread::sleep(Duration::from_millis(ms));
}
