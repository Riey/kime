use std::error::Error;
use std::os::fd::{AsFd, AsRawFd};
use std::time::{Duration, Instant};

use kime_engine_core::{
    load_engine_config_from_config_dir, Config, InputEngine, InputResult, Key, KeyCode,
    ModifierState,
};

use mio::{unix::SourceFd, Events as MioEvents, Interest, Poll, Token};
use timerfd_mio::TimerFd;

use wayland_client::{
    delegate_noop, event_created_child,
    protocol::{
        wl_keyboard::{self, KeyState, KeymapFormat, WlKeyboard},
        wl_registry::{self, WlRegistry},
        wl_seat::{self, WlSeat},
    },
    Connection, Dispatch, EventQueue, Proxy, QueueHandle, WEnum,
};

use wayland_protocols_misc::zwp_input_method_v2::client::{
    zwp_input_method_keyboard_grab_v2::{self, ZwpInputMethodKeyboardGrabV2},
    zwp_input_method_manager_v2::ZwpInputMethodManagerV2,
    zwp_input_method_v2::{self, ZwpInputMethodV2},
};

use wayland_protocols::wp::input_method::zv1::client::{
    zwp_input_method_context_v1::{self, ZwpInputMethodContextV1},
    zwp_input_method_v1::{self, ZwpInputMethodV1},
};

use wayland_protocols_misc::zwp_virtual_keyboard_v1::client::{
    zwp_virtual_keyboard_manager_v1::ZwpVirtualKeyboardManagerV1,
    zwp_virtual_keyboard_v1::ZwpVirtualKeyboardV1,
};

use crate::{PressState, RepeatInfo};

use xkbcommon::xkb::{
    Context as XkbContext, Keycode, Keymap, CONTEXT_NO_FLAGS, KEYMAP_COMPILE_NO_FLAGS,
    KEYMAP_FORMAT_TEXT_V1,
};

const ZWP_TEXT_INPUT_V1_PREEDIT_STYLE_UNDERLINE: u32 = 5;

/// Registry state for binding globals
struct Globals {
    seat: Option<WlSeat>,
    im_manager_v2: Option<ZwpInputMethodManagerV2>,
    vk_manager: Option<ZwpVirtualKeyboardManagerV1>,
    im_v1: Option<ZwpInputMethodV1>,
}

/// Input method v2 state
struct InputMethodV2State {
    im: ZwpInputMethodV2,
    #[allow(dead_code)]
    grab: ZwpInputMethodKeyboardGrabV2,
    vk: ZwpVirtualKeyboardV1,
    pending_activate: bool,
    pending_deactivate: bool,
    current_activate: bool,
    current_deactivate: bool,
    grab_activate: bool,
    keymap_init: bool,
}

impl InputMethodV2State {
    /// Forward a key to the virtual keyboard, unless no keymap was uploaded to
    /// it yet — the compositor rejects keys before a keymap with a protocol
    /// error, and a seat whose keyboard has no keymap produces no real keys
    /// anyway. See the `Keymap` event handler for how that happens.
    fn vk_key(&self, time: u32, key: u32, key_state: u32) {
        if self.keymap_init {
            self.vk.key(time, key, key_state);
        }
    }

    /// Forward a modifier state to the virtual keyboard; rejected before a
    /// keymap just like [`Self::vk_key`].
    fn vk_modifiers(&self, depressed: u32, latched: u32, locked: u32, group: u32) {
        if self.keymap_init {
            self.vk.modifiers(depressed, latched, locked, group);
        }
    }
}

/// Input method v1 state
struct InputMethodV1State {
    im_ctx: Option<ZwpInputMethodContextV1>,
    keyboard: Option<WlKeyboard>,
    keymap: Option<Keymap>,
    grab_activate: bool,
}

/// Main application state
pub struct AppState {
    config: Config,
    engine: InputEngine,
    mod_state: ModifierState,
    numlock: bool,
    engine_ready: bool,
    serial: u32,
    timer: TimerFd,
    repeat_state: Option<(RepeatInfo, PressState)>,
    /// `true` when a repeat tick of the currently pressed key was bypassed to
    /// the client as a synthetic press, i.e. a synthetic press is outstanding.
    /// Cleared on a new press and on the real key release.
    repeat_bypassed: bool,
    /// The preedit string last sent to the client, empty when the client has
    /// no preedit. Reset whenever a text input is activated or deactivated,
    /// since the client drops its preedit with the old context.
    last_preedit: String,
    should_exit: bool,

    // Wayland state
    globals: Globals,
    im_v2: Option<InputMethodV2State>,
    im_v1: Option<InputMethodV1State>,
}

impl AppState {
    pub fn new(_conn: &Connection) -> Self {
        let config = load_engine_config_from_config_dir().unwrap_or_default();
        let timer = TimerFd::new().expect("Initialize timer");

        Self {
            engine: InputEngine::new(&config),
            config,
            mod_state: ModifierState::empty(),
            numlock: false,
            engine_ready: true,
            serial: 0,
            timer,
            // For v1, provide default repeat info since older protocols might not provide it
            repeat_state: Some((
                RepeatInfo {
                    rate: 20,
                    delay: 400,
                },
                PressState::NotPressing,
            )),
            repeat_bypassed: false,
            last_preedit: String::new(),
            should_exit: false,
            globals: Globals {
                seat: None,
                im_manager_v2: None,
                vk_manager: None,
                im_v1: None,
            },
            im_v2: None,
            im_v1: None,
        }
    }

    pub fn has_input_method_v2(&self) -> bool {
        self.globals.im_manager_v2.is_some()
            && self.globals.vk_manager.is_some()
            && self.globals.seat.is_some()
    }

    pub fn has_input_method_v1(&self) -> bool {
        self.globals.im_v1.is_some()
    }

    pub fn has_vk_manager(&self) -> bool {
        self.globals.vk_manager.is_some()
    }

    pub fn has_seat(&self) -> bool {
        self.globals.seat.is_some()
    }

    pub fn setup_input_method_v2(&mut self, qh: &QueueHandle<Self>) -> Result<(), Box<dyn Error>> {
        let seat = self.globals.seat.as_ref().ok_or("No seat")?;
        let im_manager = self
            .globals
            .im_manager_v2
            .as_ref()
            .ok_or("No input method manager v2")?;
        let vk_manager = self
            .globals
            .vk_manager
            .as_ref()
            .ok_or("No virtual keyboard manager")?;

        let im = im_manager.get_input_method(seat, qh, ());
        let grab = im.grab_keyboard(qh, ());
        let vk = vk_manager.create_virtual_keyboard(seat, qh, ());

        self.im_v2 = Some(InputMethodV2State {
            im,
            grab,
            vk,
            pending_activate: false,
            pending_deactivate: false,
            current_activate: false,
            current_deactivate: false,
            grab_activate: false,
            keymap_init: false,
        });

        Ok(())
    }

    pub fn setup_input_method_v1(&mut self, _qh: &QueueHandle<Self>) -> Result<(), Box<dyn Error>> {
        // v1 is already bound through registry in Dispatch<WlRegistry>
        // The actual context activation happens via the Activate event
        if self.globals.im_v1.is_none() {
            return Err("No input method v1".into());
        }

        self.im_v1 = Some(InputMethodV1State {
            im_ctx: None,
            keyboard: None,
            keymap: None,
            grab_activate: false,
        });

        Ok(())
    }

    pub fn run_event_loop(mut self, conn: &Connection, mut event_queue: EventQueue<Self>) {
        let mut poll = Poll::new().expect("Initialize epoll()");
        let registry = poll.registry();

        const POLL_WAYLAND: Token = Token(0);
        registry
            .register(
                &mut SourceFd(&conn.as_fd().as_raw_fd()),
                POLL_WAYLAND,
                Interest::READABLE | Interest::WRITABLE,
            )
            .expect("Register wayland socket to the epoll()");

        const POLL_TIMER: Token = Token(1);
        registry
            .register(&mut self.timer, POLL_TIMER, Interest::READABLE)
            .expect("Register timer to the epoll()");

        let mut events = MioEvents::with_capacity(1024);

        loop {
            // Flush pending writes first
            if let Err(e) = conn.flush() {
                if let wayland_client::backend::WaylandError::Io(ref io_err) = e {
                    if io_err.kind() != std::io::ErrorKind::WouldBlock {
                        log::error!("Connection flush error: {}", e);
                        break;
                    }
                } else {
                    log::error!("Connection flush error: {}", e);
                    break;
                }
            }

            // Sleep until next event
            if let Err(e) = poll.poll(&mut events, None) {
                if e.kind() == std::io::ErrorKind::Interrupted {
                    continue;
                }
                log::error!("Poll error: {}", e);
                break;
            }

            for event in &events {
                match event.token() {
                    POLL_WAYLAND => {
                        // Read events - prepare_read returns Option<ReadEventsGuard>
                        if let Some(guard) = conn.prepare_read() {
                            if let Err(e) = guard.read() {
                                if let wayland_client::backend::WaylandError::Io(ref io_err) = e {
                                    if io_err.kind() != std::io::ErrorKind::WouldBlock {
                                        log::error!("Read error: {}", e);
                                        return;
                                    }
                                } else {
                                    log::error!("Read error: {}", e);
                                    return;
                                }
                            }
                        }

                        // Dispatch pending events
                        if let Err(e) = event_queue.dispatch_pending(&mut self) {
                            log::error!("Dispatch error: {}", e);
                            return;
                        }

                        // Check if we should exit (e.g., Unavailable event)
                        if self.should_exit {
                            log::info!("Exiting due to should_exit flag");
                            return;
                        }
                    }
                    POLL_TIMER => {
                        if let Err(e) = self.handle_timer_ev() {
                            log::error!("Timer error: {}", e);
                            return;
                        }
                    }
                    _ => unreachable!(),
                }
            }
        }
    }

    fn handle_timer_ev(&mut self) -> std::io::Result<()> {
        let overrun_count = self.timer.read()?;
        if overrun_count == 0 {
            return Ok(());
        }
        if overrun_count != 1 {
            log::warn!("Some timer events were not properly handled!");
        }

        if let Some((
            info,
            PressState::Pressing {
                pressed_at,
                ref mut is_repeating,
                key,
                wayland_time,
            },
        )) = self.repeat_state
        {
            // Check if key should repeat (v1 uses keymap, v2 always repeats)
            let should_start_repeat = if let Some(ref im_state) = self.im_v1 {
                im_state
                    .keymap
                    .as_ref()
                    .map_or(true, |km| km.key_repeats(Keycode::new(key + 8)))
            } else {
                true
            };

            if !*is_repeating && should_start_repeat {
                log::trace!("Start repeating {}", key);
                let interval = Duration::from_secs_f64(1.0 / info.rate as f64);
                self.timer.set_timeout_interval(interval, interval)?;
                *is_repeating = true;
            }

            // Emit key repeat event
            let elapsed_ms = pressed_at.elapsed().as_millis() as u32;
            let time = wayland_time.wrapping_add(elapsed_ms);

            // Handle v2
            let v2_grab_activate = self
                .im_v2
                .as_ref()
                .map(|s| s.grab_activate)
                .unwrap_or(false);
            if v2_grab_activate {
                let hwcode = (key + 8) as u16;
                if let Some(code) = KeyCode::from_hardware_code(hwcode, self.numlock) {
                    let ret = self
                        .engine
                        .press_key(Key::new(code, self.mod_state), &self.config);
                    let bypassed = self.process_input_result_v2(ret);
                    if bypassed {
                        // The engine no longer consumes the repeats of this key
                        // (e.g. Backspace after the preedit is emptied), so keep
                        // repeating it for the client ourselves. Send a release
                        // before every press after the first so the client sees
                        // one keystroke per tick instead of a held key that
                        // would restart its own delayed repeat.
                        if let Some(ref im_state) = self.im_v2 {
                            if self.repeat_bypassed {
                                im_state.vk_key(time, key, KeyState::Released as u32);
                            }
                            im_state.vk_key(time, key, KeyState::Pressed as u32);
                        }
                        self.repeat_bypassed = true;
                    }
                }
            }

            // Handle v1
            let v1_grab_activate = self
                .im_v1
                .as_ref()
                .map(|s| s.grab_activate)
                .unwrap_or(false);
            if v1_grab_activate {
                let hwcode = (key + 8) as u16;
                if let Some(code) = KeyCode::from_hardware_code(hwcode, self.numlock) {
                    let ret = self
                        .engine
                        .press_key(Key::new(code, self.mod_state), &self.config);
                    let bypassed = self.process_input_result_v1(ret);
                    if bypassed {
                        if self.repeat_bypassed {
                            self.key_v1(time, key, KeyState::Released);
                        }
                        self.key_v1(time, key, KeyState::Pressed);
                        self.repeat_bypassed = true;
                    }
                }
            }
        }

        Ok(())
    }

    fn process_input_result_v2(&mut self, ret: InputResult) -> bool {
        if ret.contains(InputResult::NOT_READY) {
            self.engine_ready = false;
        }

        if ret.contains(InputResult::LANGUAGE_CHANGED) {
            self.engine.update_layout_state().ok();
        }

        if let Some(ref im_state) = self.im_v2 {
            // `commit_str()`/`preedit_str()` are zero-copy borrows from the
            // engine: the whole should-we-send decision runs on them, so a key
            // that changes nothing (lone modifiers, volume keys, ... bypassed
            // while composing — the engine still reports `HAS_PREEDIT` for
            // those) allocates nothing and sends nothing (issue #781). Owned
            // strings are made only for the protocol calls that demand them.
            //
            // `preedit_str` rebuilds into the engine's buffer under `&mut`, so
            // its borrow cannot overlap `commit_str`'s — the commit text is
            // therefore only borrowed right at its protocol call.
            // HAS_COMMIT is set iff the commit buffer is non-empty
            // (InputEngine::current_result), so the flag alone decides.
            let has_commit = ret.contains(InputResult::HAS_COMMIT);
            let preedit = if ret.contains(InputResult::HAS_PREEDIT) {
                self.engine.preedit_str()
            } else {
                ""
            };

            // A transaction is sent only when it carries something new. Text
            // to commit always does; a preedit only when it differs from the
            // one the client already shows — but an unchanged preedit is
            // staged again whenever there is a commit string: `commit` resets
            // the pending state, so the preedit has to be part of every
            // transaction that survives it.
            if has_commit || preedit != self.last_preedit {
                if !preedit.is_empty() {
                    im_state
                        .im
                        .set_preedit_string(preedit.into(), 0, preedit.len() as i32);
                } else if !self.last_preedit.is_empty() {
                    im_state.im.set_preedit_string(String::new(), -1, -1);
                }

                // Remember what the client will show, reusing the buffer.
                // Also the last use of `preedit`, releasing the engine borrow.
                self.last_preedit.clear();
                self.last_preedit.push_str(preedit);

                if has_commit {
                    im_state.im.commit_string(self.engine.commit_str().into());
                }

                // Under the double-buffered input-method-v2 -> text-input-v3
                // semantics a bare `commit` applies an "empty preedit, no
                // commit string" state and yields a spurious `done` at the
                // client, which Firefox/Chromium interpret as an empty
                // composition that replaces the current selection (issue
                // #714), so `commit` is only ever sent with state above it.
                im_state.im.commit(self.serial);
            }
        }

        if ret.contains(InputResult::HAS_COMMIT) {
            self.engine.clear_commit();
        }

        !ret.contains(InputResult::CONSUMED)
    }

    fn process_input_result_v1(&mut self, ret: InputResult) -> bool {
        if ret.contains(InputResult::NOT_READY) {
            self.engine_ready = false;
        }

        if ret.contains(InputResult::LANGUAGE_CHANGED) {
            self.engine.update_layout_state().ok();
        }

        if let Some(ref im_state) = self.im_v1 {
            if let Some(ref im_ctx) = im_state.im_ctx {
                // Zero-copy borrows with the same discipline as the v2 path.
                // v1 must send `commit_string` (which clears the client's
                // preedit) BEFORE re-sending the preedit, so the preedit is
                // borrowed twice: once for the decision, once for the send —
                // still no allocation on the nothing-changed path.
                // HAS_COMMIT implies a non-empty commit buffer, as in v2.
                let has_commit = ret.contains(InputResult::HAS_COMMIT);
                let preedit_changed = {
                    let preedit = if ret.contains(InputResult::HAS_PREEDIT) {
                        self.engine.preedit_str()
                    } else {
                        ""
                    };
                    preedit != self.last_preedit
                };

                // Same rule as the v2 path: skip a preedit identical to the one
                // the client already shows (issue #781), but always re-send it
                // together with a commit string, which clears the preedit at
                // the client.
                if has_commit || preedit_changed {
                    if has_commit {
                        im_ctx.commit_string(self.serial, self.engine.commit_str().into());
                    }

                    let preedit = if ret.contains(InputResult::HAS_PREEDIT) {
                        self.engine.preedit_str()
                    } else {
                        ""
                    };
                    if !preedit.is_empty() {
                        let len = preedit.len();
                        im_ctx.preedit_cursor(len as i32);
                        im_ctx.preedit_styling(
                            0,
                            len as u32,
                            ZWP_TEXT_INPUT_V1_PREEDIT_STYLE_UNDERLINE,
                        );
                        im_ctx.preedit_string(self.serial, preedit.into(), preedit.into());
                    } else if !self.last_preedit.is_empty() {
                        im_ctx.preedit_cursor(0);
                        im_ctx.preedit_string(self.serial, String::new(), String::new());
                    }

                    self.last_preedit.clear();
                    self.last_preedit.push_str(preedit);
                }
            }
        }

        if ret.contains(InputResult::HAS_COMMIT) {
            self.engine.clear_commit();
        }

        !ret.contains(InputResult::CONSUMED)
    }

    fn key_v1(&mut self, time: u32, key: u32, state: KeyState) {
        if let Some(ref im_state) = self.im_v1 {
            if let Some(ref im_ctx) = im_state.im_ctx {
                im_ctx.key(self.serial, time, key, state as u32);
            }
        }
    }

    fn modifiers_v1(
        &mut self,
        mods_depressed: u32,
        mods_latched: u32,
        mods_locked: u32,
        group: u32,
    ) {
        if let Some(ref im_state) = self.im_v1 {
            if let Some(ref im_ctx) = im_state.im_ctx {
                im_ctx.modifiers(
                    self.serial,
                    mods_depressed,
                    mods_latched,
                    mods_locked,
                    group,
                );
            }
        }
    }
}

// Registry dispatch
impl Dispatch<WlRegistry, ()> for AppState {
    fn event(
        state: &mut Self,
        registry: &WlRegistry,
        event: wl_registry::Event,
        _data: &(),
        _conn: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        if let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
        {
            log::debug!("Registry global [{}]: {} v{}", name, interface, version);
            match interface.as_str() {
                "wl_seat" => {
                    log::debug!("Binding wl_seat");
                    let seat = registry.bind::<WlSeat, _, _>(name, version.min(7), qh, ());
                    state.globals.seat = Some(seat);
                }
                "zwp_input_method_manager_v2" => {
                    log::debug!("Binding zwp_input_method_manager_v2");
                    let mgr = registry.bind::<ZwpInputMethodManagerV2, _, _>(
                        name,
                        version.min(1),
                        qh,
                        (),
                    );
                    state.globals.im_manager_v2 = Some(mgr);
                }
                "zwp_virtual_keyboard_manager_v1" => {
                    log::debug!("Binding zwp_virtual_keyboard_manager_v1");
                    let mgr = registry.bind::<ZwpVirtualKeyboardManagerV1, _, _>(
                        name,
                        version.min(1),
                        qh,
                        (),
                    );
                    state.globals.vk_manager = Some(mgr);
                }
                "zwp_input_method_v1" => {
                    log::debug!("Binding zwp_input_method_v1");
                    let im = registry.bind::<ZwpInputMethodV1, _, _>(name, version.min(1), qh, ());
                    state.globals.im_v1 = Some(im);
                }
                _ => {}
            }
        }
    }
}

// Seat dispatch (no-op, just need to bind)
impl Dispatch<WlSeat, ()> for AppState {
    fn event(
        _state: &mut Self,
        _seat: &WlSeat,
        _event: wl_seat::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
    }
}

// Input method manager v2 dispatch (no events)
delegate_noop!(AppState: ignore ZwpInputMethodManagerV2);
delegate_noop!(AppState: ignore ZwpVirtualKeyboardManagerV1);
delegate_noop!(AppState: ignore ZwpVirtualKeyboardV1);

// Input method v2 dispatch
impl Dispatch<ZwpInputMethodV2, ()> for AppState {
    fn event(
        state: &mut Self,
        _im: &ZwpInputMethodV2,
        event: zwp_input_method_v2::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        log::trace!("input_method_v2 event: {:?}", event);
        match event {
            zwp_input_method_v2::Event::Activate => {
                if let Some(ref mut im_state) = state.im_v2 {
                    im_state.pending_activate = true;
                }
            }
            zwp_input_method_v2::Event::Deactivate => {
                if let Some(ref mut im_state) = state.im_v2 {
                    im_state.pending_deactivate = true;
                }
            }
            zwp_input_method_v2::Event::Unavailable => {
                log::error!("Receive Unavailable event, is another server already running?");
                state.should_exit = true;
            }
            zwp_input_method_v2::Event::Done => {
                state.serial += 1;

                let (should_activate, should_deactivate) = {
                    if let Some(ref im_state) = state.im_v2 {
                        let activate = !im_state.current_activate && im_state.pending_activate;
                        let deactivate =
                            !im_state.current_deactivate && im_state.pending_deactivate;
                        (activate, deactivate)
                    } else {
                        (false, false)
                    }
                };

                if should_activate {
                    // The text input starts out with no preedit, so nothing we
                    // sent to the previous one may suppress the next preedit.
                    state.last_preedit.clear();
                    state.engine.update_layout_state().ok();
                    if !state.engine_ready && state.engine.check_ready() {
                        let ret = state.engine.end_ready();
                        state.process_input_result_v2(ret);
                        state.engine_ready = true;
                    }
                    if let Some(ref mut im_state) = state.im_v2 {
                        im_state.grab_activate = true;
                    }
                } else if should_deactivate {
                    if state.engine_ready {
                        state.engine.reset();
                    }
                    state.last_preedit.clear();
                    if let Some(ref mut im_state) = state.im_v2 {
                        im_state.grab_activate = false;
                    }
                    let _ = state.timer.set_timeout_oneshot(Duration::ZERO);
                    if let Some((_, ref mut ps)) = state.repeat_state {
                        *ps = PressState::NotPressing;
                    }
                    state.repeat_bypassed = false;
                }

                if let Some(ref mut im_state) = state.im_v2 {
                    im_state.current_activate = im_state.pending_activate;
                    im_state.current_deactivate = im_state.pending_deactivate;
                    im_state.pending_activate = false;
                    im_state.pending_deactivate = false;
                }
            }
            _ => {}
        }
    }
}

// Keyboard grab v2 dispatch
impl Dispatch<ZwpInputMethodKeyboardGrabV2, ()> for AppState {
    fn event(
        state: &mut Self,
        _grab: &ZwpInputMethodKeyboardGrabV2,
        event: zwp_input_method_keyboard_grab_v2::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        match event {
            zwp_input_method_keyboard_grab_v2::Event::Keymap { format, fd, size } => {
                if let Some(ref mut im_state) = state.im_v2 {
                    if !im_state.keymap_init {
                        // A seat whose keyboard has no keymap is reported as
                        // `no_keymap` with size 0. The virtual keyboard can't mmap
                        // that, so forwarding it kills the connection; wait for a
                        // real keymap instead, which the compositor sends once the
                        // seat has a keyboard with one.
                        if matches!(format, WEnum::Value(KeymapFormat::XkbV1)) && size > 0 {
                            im_state.vk.keymap(format.into(), fd.as_fd(), size);
                            im_state.keymap_init = true;
                        } else {
                            log::debug!(
                                "Ignoring empty keymap (format={:?}, size={})",
                                format,
                                size
                            );
                        }
                    }
                }
            }
            zwp_input_method_keyboard_grab_v2::Event::Key {
                time,
                key,
                state: key_state,
                ..
            } => {
                let is_pressed = matches!(key_state, WEnum::Value(KeyState::Pressed))
                    || key_state == WEnum::Value(KeyState::Pressed)
                    || matches!(key_state, WEnum::Value(KeyState::Repeated))
                    || matches!(&key_state, WEnum::Unknown(2)); // Repeated

                let grab_activate = state
                    .im_v2
                    .as_ref()
                    .map(|s| s.grab_activate)
                    .unwrap_or(false);

                if is_pressed && grab_activate {
                    let hwcode = (key + 8) as u16;
                    if let Some(code) = KeyCode::from_hardware_code(hwcode, state.numlock) {
                        let ret = state
                            .engine
                            .press_key(Key::new(code, state.mod_state), &state.config);

                        let bypassed = state.process_input_result_v2(ret);

                        if bypassed {
                            if let Some(ref im_state) = state.im_v2 {
                                im_state.vk_key(time, key, key_state.into());
                            }
                        } else {
                            // Handle key repeat
                            if let Some((info, ref mut press_state)) = state.repeat_state {
                                if !press_state.is_pressing(key) {
                                    let duration = Duration::from_millis(info.delay as u64);
                                    if let Err(e) = state.timer.set_timeout_oneshot(duration) {
                                        log::warn!("failed to set repeat timer: {}", e);
                                    }
                                    *press_state = PressState::Pressing {
                                        pressed_at: Instant::now(),
                                        is_repeating: false,
                                        key,
                                        wayland_time: time,
                                    };
                                    state.repeat_bypassed = false;
                                }
                            }
                        }
                    } else if let Some(ref im_state) = state.im_v2 {
                        im_state.vk_key(time, key, key_state.into());
                    }
                } else if matches!(key_state, WEnum::Value(KeyState::Released)) {
                    if let Some((.., ref mut press_state)) = state.repeat_state {
                        if press_state.is_pressing(key) {
                            let _ = state.timer.set_timeout_oneshot(Duration::ZERO);
                            *press_state = PressState::NotPressing;
                            // The release below balances any synthetic press
                            // emitted for a bypassed repeat tick.
                            state.repeat_bypassed = false;
                        }
                    }
                    if let Some(ref im_state) = state.im_v2 {
                        im_state.vk_key(time, key, key_state.into());
                    }
                } else {
                    // is_pressed && !grab_activate - bypass
                    if let Some(ref im_state) = state.im_v2 {
                        im_state.vk_key(time, key, key_state.into());
                    }
                }
            }
            zwp_input_method_keyboard_grab_v2::Event::Modifiers {
                mods_depressed,
                mods_latched,
                mods_locked,
                group,
                ..
            } => {
                state.mod_state = ModifierState::empty();
                if mods_depressed & 0x1 != 0 {
                    state.mod_state |= ModifierState::SHIFT;
                }
                if mods_depressed & 0x4 != 0 {
                    state.mod_state |= ModifierState::CONTROL;
                }
                if mods_depressed & 0x8 != 0 {
                    state.mod_state |= ModifierState::ALT;
                }
                if mods_depressed & 0x40 != 0 {
                    state.mod_state |= ModifierState::SUPER;
                }
                state.numlock = mods_depressed & 0x10 != 0;

                if let Some(ref im_state) = state.im_v2 {
                    im_state.vk_modifiers(mods_depressed, mods_latched, mods_locked, group);
                }
            }
            zwp_input_method_keyboard_grab_v2::Event::RepeatInfo { rate, delay } => {
                state.repeat_state = if rate == 0 {
                    None
                } else {
                    let info = RepeatInfo { rate, delay };
                    let press_state = state
                        .repeat_state
                        .map(|pair| pair.1)
                        .unwrap_or(PressState::NotPressing);
                    Some((info, press_state))
                };
            }
            _ => {}
        }
    }
}

// Input method v1 dispatch
impl Dispatch<ZwpInputMethodV1, ()> for AppState {
    event_created_child!(AppState, ZwpInputMethodV1, [
        // opcode 0 = activate event, creates ZwpInputMethodContextV1
        0 => (ZwpInputMethodContextV1, ())
    ]);

    fn event(
        state: &mut Self,
        _im: &ZwpInputMethodV1,
        event: zwp_input_method_v1::Event,
        _data: &(),
        _conn: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        match event {
            zwp_input_method_v1::Event::Activate { id: im_ctx } => {
                log::trace!("input_method_v1: Activate");
                // Fresh context: it shows no preedit yet.
                state.last_preedit.clear();
                state.engine.update_layout_state().ok();
                if !state.engine_ready && state.engine.check_ready() {
                    let ret = state.engine.end_ready();
                    state.process_input_result_v1(ret);
                    state.engine_ready = true;
                }

                // Grab keyboard
                let keyboard = im_ctx.grab_keyboard(qh, ());

                if let Some(ref mut im_state) = state.im_v1 {
                    im_state.im_ctx = Some(im_ctx);
                    im_state.keyboard = Some(keyboard);
                    im_state.grab_activate = true;
                }
            }
            zwp_input_method_v1::Event::Deactivate { context: im_ctx } => {
                log::trace!("input_method_v1: Deactivate");
                if state.engine_ready {
                    state.engine.reset();
                }
                state.last_preedit.clear();

                // Stop repeating
                let _ = state.timer.set_timeout_oneshot(Duration::ZERO);
                if let Some((_, ref mut press_state)) = state.repeat_state {
                    *press_state = PressState::NotPressing;
                }
                state.repeat_bypassed = false;

                im_ctx.destroy();

                if let Some(ref mut im_state) = state.im_v1 {
                    im_state.im_ctx = None;
                    if let Some(ref keyboard) = im_state.keyboard {
                        if keyboard.version() >= wl_keyboard::REQ_RELEASE_SINCE {
                            keyboard.release();
                        }
                    }
                    im_state.keyboard = None;
                    im_state.grab_activate = false;
                }
            }
            _ => {}
        }
    }
}

// Input method context v1 dispatch
impl Dispatch<ZwpInputMethodContextV1, ()> for AppState {
    fn event(
        state: &mut Self,
        _im_ctx: &ZwpInputMethodContextV1,
        event: zwp_input_method_context_v1::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        match event {
            zwp_input_method_context_v1::Event::CommitState { serial } => {
                state.serial = serial;
            }
            _ => {}
        }
    }
}

// WlKeyboard dispatch for v1
impl Dispatch<WlKeyboard, ()> for AppState {
    fn event(
        state: &mut Self,
        _keyboard: &WlKeyboard,
        event: wl_keyboard::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        match event {
            wl_keyboard::Event::Keymap { format, fd, size } => {
                if let WEnum::Value(KeymapFormat::XkbV1) = format {
                    // fd is already OwnedFd in wayland-client 0.31, use it directly
                    if let Some(ref mut im_state) = state.im_v1 {
                        im_state.keymap = unsafe {
                            Keymap::new_from_fd(
                                &XkbContext::new(CONTEXT_NO_FLAGS),
                                fd,
                                size as usize,
                                KEYMAP_FORMAT_TEXT_V1,
                                KEYMAP_COMPILE_NO_FLAGS,
                            )
                            .unwrap_or(None)
                        };
                    }
                }
            }
            wl_keyboard::Event::Key {
                time,
                key,
                state: key_state,
                ..
            } => {
                let is_pressed = matches!(key_state, WEnum::Value(KeyState::Pressed))
                    || matches!(key_state, WEnum::Value(KeyState::Repeated))
                    || matches!(&key_state, WEnum::Unknown(2)); // Repeated

                let grab_activate = state
                    .im_v1
                    .as_ref()
                    .map(|s| s.grab_activate)
                    .unwrap_or(false);

                if is_pressed && grab_activate {
                    let hwcode = (key + 8) as u16;
                    if let Some(code) = KeyCode::from_hardware_code(hwcode, state.numlock) {
                        let ret = state
                            .engine
                            .press_key(Key::new(code, state.mod_state), &state.config);

                        let bypassed = state.process_input_result_v1(ret);

                        if bypassed {
                            if let WEnum::Value(ks) = key_state {
                                state.key_v1(time, key, ks);
                            }
                        } else {
                            // Handle key repeat
                            if let Some((info, ref mut press_state)) = state.repeat_state {
                                if !press_state.is_pressing(key) {
                                    let duration = Duration::from_millis(info.delay as u64);
                                    if let Err(e) = state.timer.set_timeout_oneshot(duration) {
                                        log::warn!("failed to set repeat timer: {}", e);
                                    }
                                    *press_state = PressState::Pressing {
                                        pressed_at: Instant::now(),
                                        is_repeating: false,
                                        key,
                                        wayland_time: time,
                                    };
                                    state.repeat_bypassed = false;
                                }
                            }
                        }
                    } else if let WEnum::Value(ks) = key_state {
                        state.key_v1(time, key, ks);
                    }
                } else if matches!(key_state, WEnum::Value(KeyState::Released)) {
                    if let Some((.., ref mut press_state)) = state.repeat_state {
                        if press_state.is_pressing(key) {
                            let _ = state.timer.set_timeout_oneshot(Duration::ZERO);
                            *press_state = PressState::NotPressing;
                            // The release below balances any synthetic press
                            // emitted for a bypassed repeat tick.
                            state.repeat_bypassed = false;
                        }
                    }
                    if let WEnum::Value(ks) = key_state {
                        state.key_v1(time, key, ks);
                    }
                } else if let WEnum::Value(ks) = key_state {
                    // is_pressed && !grab_activate - bypass
                    state.key_v1(time, key, ks);
                }
            }
            wl_keyboard::Event::Modifiers {
                mods_depressed,
                mods_latched,
                mods_locked,
                group,
                ..
            } => {
                state.mod_state = ModifierState::empty();
                if mods_depressed & 0x1 != 0 {
                    state.mod_state |= ModifierState::SHIFT;
                }
                if mods_depressed & 0x4 != 0 {
                    state.mod_state |= ModifierState::CONTROL;
                }
                if mods_depressed & 0x8 != 0 {
                    state.mod_state |= ModifierState::ALT;
                }
                if mods_depressed & 0x40 != 0 {
                    state.mod_state |= ModifierState::SUPER;
                }
                state.numlock = mods_depressed & 0x10 != 0;

                state.modifiers_v1(mods_depressed, mods_latched, mods_locked, group);
            }
            wl_keyboard::Event::RepeatInfo { rate, delay } => {
                state.repeat_state = if rate == 0 {
                    None
                } else {
                    let info = RepeatInfo { rate, delay };
                    let press_state = state
                        .repeat_state
                        .map(|pair| pair.1)
                        .unwrap_or(PressState::NotPressing);
                    Some((info, press_state))
                };
            }
            _ => {}
        }
    }
}
