//! End-to-end GUI test harness for kime.
//!
//! Tests live in `tests/*.rs`, are all `#[ignore]`, and run with
//! `cargo test -p kime-e2e -- --ignored --test-threads=1` (or `tests/e2e/run.sh`).
//!
//! Hard rules baked into this harness:
//! - Never touch the user's live session: every spawned process gets
//!   [`envs::clean_cmd`] (env_clear + allowlist), never the inherited
//!   `WAYLAND_DISPLAY`/`DISPLAY`.
//! - Never kill by name: [`proc::Proc`] is an RAII guard that kills only the
//!   exact pid it spawned.
//! - `LD_LIBRARY_PATH` always starts with the local `target/debug`
//!   (the system `libkime_engine.so` is stale on dev machines), and immodule
//!   tests must verify the local module loaded via [`stage::maps_check`].
//! - On Wayland, the [`inject::VkbdInjector`] must be constructed BEFORE
//!   [`kime::KimeWayland`] (kime crashes forwarding an empty keymap from a
//!   keyboardless seat).

pub mod cc;
pub mod envs;
pub mod inject;
pub mod kime;
pub mod paths;
pub mod probes;
pub mod proc;
pub mod stage;
pub mod sway;
pub mod wldebug;
pub mod x11;

/// Harness-wide result type: error strings say which process/file failed.
pub type Result<T> = std::result::Result<T, String>;
