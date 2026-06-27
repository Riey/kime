//! `org.freedesktop.IBus.Engine` implementation.
//!
//! One [`IBusEngine`] instance is served per IBus input context (created by the
//! factory's `CreateEngine`). It owns its own [`InputEngine`] and shares the
//! loaded [`Config`] through an `Arc`.
//!
//! The key/preedit/commit logic mirrors the xim and wayland frontends exactly:
//! we feed the X11 hardware keycode to [`InputEngine::press_key_code`] and act on
//! the returned [`InputResult`] bitflags.

use std::sync::Arc;

use kime_engine_core::{Config, InputEngine, InputResult, ModifierState};
use zbus::{interface, zvariant::Value, Connection};

use crate::ibus_types::ibus_text;

/// IBus modifier mask bits (from `ibustypes.h`). These mirror the X11 modifier
/// masks that IBus forwards in `ProcessKeyEvent`'s `state` argument.
const IBUS_SHIFT_MASK: u32 = 1 << 0;
const IBUS_CONTROL_MASK: u32 = 1 << 2;
const IBUS_MOD1_MASK: u32 = 1 << 3; // Alt
const IBUS_MOD2_MASK: u32 = 1 << 4; // NumLock
const IBUS_MOD4_MASK: u32 = 1 << 6; // commonly the physical Super key
const IBUS_SUPER_MASK: u32 = 1 << 26; // virtual Super modifier
const IBUS_RELEASE_MASK: u32 = 1 << 30;

const ENGINE_INTERFACE: &str = "org.freedesktop.IBus.Engine";

pub struct IBusEngine {
    engine: InputEngine,
    config: Arc<Config>,
    conn: Connection,
    /// D-Bus object path this engine is served at; used as the signal sender path.
    path: String,
    engine_ready: bool,
    /// Byte length of the last preedit we sent, so we only emit a "hide" once.
    last_preedit_len: usize,
}

impl IBusEngine {
    pub fn new(conn: Connection, path: String, config: Arc<Config>) -> Self {
        Self {
            engine: InputEngine::new(&config),
            config,
            conn,
            path,
            engine_ready: true,
            last_preedit_len: 0,
        }
    }

    async fn commit_text(&self, text: &str) {
        let v = ibus_text(text);
        if let Err(e) = self
            .conn
            .emit_signal(
                None::<&str>,
                self.path.as_str(),
                ENGINE_INTERFACE,
                "CommitText",
                &(v,),
            )
            .await
        {
            log::warn!("Failed to emit CommitText: {}", e);
        }
    }

    async fn update_preedit(&self, text: &str, cursor_pos: u32, visible: bool) {
        let v = ibus_text(text);
        if let Err(e) = self
            .conn
            .emit_signal(
                None::<&str>,
                self.path.as_str(),
                ENGINE_INTERFACE,
                "UpdatePreeditText",
                &(v, cursor_pos, visible),
            )
            .await
        {
            log::warn!("Failed to emit UpdatePreeditText: {}", e);
        }
    }

    async fn hide_preedit(&self) {
        if let Err(e) = self
            .conn
            .emit_signal(
                None::<&str>,
                self.path.as_str(),
                ENGINE_INTERFACE,
                "HidePreeditText",
                &(),
            )
            .await
        {
            log::warn!("Failed to emit HidePreeditText: {}", e);
        }
    }

    /// Interpret an [`InputResult`] the same way xim/wayland do and emit the
    /// matching IBus signals. Returns whether the key was CONSUMED.
    async fn process_result(&mut self, ret: InputResult) -> bool {
        log::trace!("InputResult: {:?}", ret);

        if ret.contains(InputResult::LANGUAGE_CHANGED) {
            self.engine.update_layout_state().ok();
        }

        // Commit finalized text first, then show the new preedit (matches xim).
        if ret.contains(InputResult::HAS_COMMIT) {
            let commit = self.engine.commit_str().to_string();
            if !commit.is_empty() {
                self.commit_text(&commit).await;
            }
            self.engine.clear_commit();
        }

        if ret.contains(InputResult::HAS_PREEDIT) {
            let preedit = self.engine.preedit_str().to_string();
            let cursor = preedit.chars().count() as u32;
            self.update_preedit(&preedit, cursor, true).await;
            self.last_preedit_len = preedit.len();
        } else if self.last_preedit_len > 0 {
            self.update_preedit("", 0, false).await;
            self.hide_preedit().await;
            self.last_preedit_len = 0;
        }

        self.engine_ready = !ret.contains(InputResult::NOT_READY);

        ret.contains(InputResult::CONSUMED)
    }

    /// Flush and clear the engine, mirroring `KimeHandler::reset` in the xim
    /// frontend: the in-progress syllable is finalized into a commit.
    async fn reset_engine(&mut self) {
        self.engine.clear_preedit();
        let commit = self.engine.commit_str().to_string();
        if !commit.is_empty() {
            self.commit_text(&commit).await;
        }
        if self.last_preedit_len > 0 {
            self.update_preedit("", 0, false).await;
            self.hide_preedit().await;
            self.last_preedit_len = 0;
        }
        self.engine.reset();
    }

    /// Shared focus-in/enable logic: refresh the global layout state and flush a
    /// pending "ready" result (math/emoji modes) if one is now available.
    async fn on_focus_in(&mut self) {
        self.engine.update_layout_state().ok();
        if !self.engine_ready && self.engine.check_ready() {
            let ret = self.engine.end_ready();
            self.process_result(ret).await;
            self.engine_ready = true;
        }
    }
}

#[interface(name = "org.freedesktop.IBus.Engine")]
impl IBusEngine {
    /// Process a key event. Returns `true` when kime consumed the key (the
    /// application must not receive it), `false` to let the application get it.
    async fn process_key_event(&mut self, _keyval: u32, keycode: u32, state: u32) -> bool {
        // Only handle key presses; releases carry IBUS_RELEASE_MASK.
        if state & IBUS_RELEASE_MASK != 0 {
            return false;
        }

        let mut mods = ModifierState::empty();
        if state & IBUS_SHIFT_MASK != 0 {
            mods |= ModifierState::SHIFT;
        }
        if state & IBUS_CONTROL_MASK != 0 {
            mods |= ModifierState::CONTROL;
        }
        if state & IBUS_MOD1_MASK != 0 {
            mods |= ModifierState::ALT;
        }
        if state & (IBUS_SUPER_MASK | IBUS_MOD4_MASK) != 0 {
            mods |= ModifierState::SUPER;
        }
        let numlock = state & IBUS_MOD2_MASK != 0;

        // IBus `keycode` is the X11 hardware keycode (evdev + 8), which is what
        // kime's keycode table expects as the hardware code.
        let ret = self
            .engine
            .press_key_code(keycode as u16, mods, numlock, &self.config);
        self.process_result(ret).await
    }

    async fn focus_in(&mut self) {
        log::trace!("FocusIn");
        self.on_focus_in().await;
    }

    async fn focus_out(&mut self) {
        log::trace!("FocusOut");
        if self.engine_ready {
            self.reset_engine().await;
        }
    }

    /// Newer focus signals (IBUS_CAP_FOCUS_ID). Same behavior as FocusIn/Out.
    async fn focus_in_id(&mut self, _object_path: String, _client: String) {
        log::trace!("FocusInId");
        self.on_focus_in().await;
    }

    async fn focus_out_id(&mut self, _object_path: String) {
        log::trace!("FocusOutId");
        if self.engine_ready {
            self.reset_engine().await;
        }
    }

    async fn enable(&mut self) {
        log::trace!("Enable");
        self.on_focus_in().await;
    }

    async fn disable(&mut self) {
        log::trace!("Disable");
        self.reset_engine().await;
    }

    async fn reset(&mut self) {
        log::trace!("Reset");
        self.reset_engine().await;
    }

    async fn destroy(&mut self) {
        log::info!("Destroy engine at {}", self.path);
        self.reset_engine().await;

        // Remove ourselves from the object server. Do it from a spawned task to
        // avoid re-entrant locking while this method still holds the interface.
        let conn = self.conn.clone();
        let path = self.path.clone();
        tokio::spawn(async move {
            if let Err(e) = conn
                .object_server()
                .remove::<IBusEngine, _>(path.as_str())
                .await
            {
                log::warn!("Failed to remove engine {}: {}", path, e);
            }
        });
    }

    // --- No-op stubs for the rest of the interface the daemon may call. ---
    // Their argument signatures must match IBus so the calls demarshal cleanly.

    fn set_cursor_location(&mut self, _x: i32, _y: i32, _w: i32, _h: i32) {}

    fn set_cursor_location_relative(&mut self, _x: i32, _y: i32, _w: i32, _h: i32) {}

    fn set_capabilities(&mut self, _caps: u32) {}

    fn set_content_type(&mut self, _purpose: u32, _hints: u32) {}

    fn set_surrounding_text(&mut self, _text: Value<'_>, _cursor_pos: u32, _anchor_pos: u32) {}

    fn property_activate(&mut self, _name: String, _state: u32) {}

    fn property_show(&mut self, _name: String) {}

    fn property_hide(&mut self, _name: String) {}

    fn candidate_clicked(&mut self, _index: u32, _button: u32, _state: u32) {}

    fn page_up(&mut self) {}

    fn page_down(&mut self) {}

    fn cursor_up(&mut self) {}

    fn cursor_down(&mut self) {}
}
