use std::error::Error;
use std::os::fd::{FromRawFd, OwnedFd};
use std::time::{Duration, Instant};

use wayland_client::{
    event_enum,
    protocol::wl_keyboard::{Event as KeyEvent, KeyState, WlKeyboard, REQ_RELEASE_SINCE},
    DispatchData, Display, EventQueue, Filter, GlobalManager, Main,
};

use wayland_protocols::unstable::input_method::v1::client::{
    zwp_input_method_context_v1::{Event as ImCtxEvent, ZwpInputMethodContextV1},
    zwp_input_method_v1::{Event as ImEvent, ZwpInputMethodV1},
};

use kime_engine_cffi::*;

use mio::{unix::SourceFd, Events as MioEvents, Interest, Poll, Token};
use mio_timerfd::{ClockId, TimerFd};
use wayland_client::protocol::wl_keyboard::KeymapFormat;
use xkbcommon::xkb::{
    Context, Keycode, Keymap, CONTEXT_NO_FLAGS, KEYMAP_COMPILE_NO_FLAGS, KEYMAP_FORMAT_TEXT_V1,
};

use crate::{PressState, RepeatInfo};

event_enum! {
    Events |
    Im => ZwpInputMethodV1,
    ImCtx => ZwpInputMethodContextV1,
    Key => WlKeyboard
}

const ZWP_TEXT_INPUT_V1_PREEDIT_STYLE_NONE: u32 = 1;
const ZWP_TEXT_INPUT_V1_PREEDIT_STYLE_UNDERLINE: u32 = 5;

struct KimeContext {
    config: Config,
    engine: InputEngine,
    mod_state: ModifierState,
    im_ctx: Option<Main<ZwpInputMethodContextV1>>,
    keyboard: Option<Main<WlKeyboard>>,
    numlock: bool,
    engine_ready: bool,
    keymap: Option<Keymap>,
    grab_activate: bool, //?
    serial: u32,
    timer: TimerFd,
    /// `None` if `KimeContext` have never received a `RepeatInfo` or repeat is disabled (i.e. rate
    /// is zero). `Some(..)` if `RepeatInfo` is known and kime-wayland started tracking the press
    /// state of keys.
    repeat_state: Option<(RepeatInfo, PressState)>,
}

impl Drop for KimeContext {
    fn drop(&mut self) {
        if let Some(im_ctx) = &mut self.im_ctx {
            im_ctx.destroy();
        }
    }
}

impl KimeContext {
    pub fn new(timer: TimerFd) -> Self {
        let config = Config::load();
        Self {
            engine: InputEngine::new(&config),
            config,
            mod_state: 0,
            serial: 0,
            numlock: false,
            engine_ready: true,
            keymap: None,
            grab_activate: false,
            im_ctx: None,
            keyboard: None,
            timer,
            // Clients with older protocols might not provide repeat info.
            // Therefore a default value is required.
            repeat_state: Some((
                RepeatInfo {
                    rate: 20,
                    delay: 400,
                },
                PressState::NotPressing,
            )),
        }
    }

    pub fn new_data<'a>(data: &'a mut DispatchData) -> &'a mut Self {
        data.get::<Self>().unwrap()
    }

    fn process_input_result(&mut self, ret: InputResult) -> bool {
        if ret & InputResult_NOT_READY != 0 {
            self.engine_ready = false;
        }

        if ret & InputResult_LANGUAGE_CHANGED != 0 {
            self.engine.update_layout_state();
        }

        if ret & InputResult_HAS_COMMIT != 0 {
            self.commit_string(self.engine.commit_str().into());
            self.engine.clear_commit();
        }

        if ret & InputResult_HAS_PREEDIT != 0 {
            let preedit = self.engine.preedit_str().into();
            self.preedit(preedit);
        } else {
            self.clear_preedit();
        }

        ret & InputResult_CONSUMED == 0
    }

    fn commit_string(&mut self, s: String) {
        if !s.is_empty() {
            if let Some(im_ctx) = &mut self.im_ctx {
                im_ctx.commit_string(self.serial, s);
            }
        }
    }

    fn clear_preedit(&mut self) {
        if let Some(im_ctx) = &mut self.im_ctx {
            im_ctx.preedit_cursor(0);
            im_ctx.preedit_styling(0, 0, ZWP_TEXT_INPUT_V1_PREEDIT_STYLE_NONE);
            im_ctx.preedit_string(self.serial, String::new(), String::new());
        }
    }

    fn preedit(&mut self, s: String) {
        if let Some(im_ctx) = &mut self.im_ctx {
            im_ctx.preedit_cursor(s.len() as _);
            im_ctx.preedit_styling(0, s.len() as _, ZWP_TEXT_INPUT_V1_PREEDIT_STYLE_UNDERLINE);
            im_ctx.preedit_string(self.serial, s.clone(), s);
        }
    }

    fn key(&mut self, time: u32, key: u32, state: KeyState) {
        if let Some(im_ctx) = &mut self.im_ctx {
            im_ctx.key(self.serial, time, key, state as _);
        }
    }

    pub fn handle_im_ctx_ev(&mut self, ev: ImCtxEvent) {
        match ev {
            ImCtxEvent::CommitState { serial } => {
                self.serial = serial;
            }
            _ => {}
        }
    }

    pub fn handle_key_ev(&mut self, ev: KeyEvent) {
        match ev {
            KeyEvent::Keymap { format, fd, size } => {
                if let KeymapFormat::XkbV1 = format {
                    unsafe {
                        self.keymap = Keymap::new_from_fd(
                            &Context::new(CONTEXT_NO_FLAGS),
                            OwnedFd::from_raw_fd(fd),
                            size as usize,
                            KEYMAP_FORMAT_TEXT_V1,
                            KEYMAP_COMPILE_NO_FLAGS,
                        )
                        .unwrap_or(None);
                    }
                } else {
                    unsafe {
                        libc::close(fd);
                    }
                }
            }
            KeyEvent::Key {
                time, key, state, ..
            } => {
                if state == KeyState::Pressed {
                    if self.grab_activate {
                        let ret = self.engine.press_key(
                            &self.config,
                            (key + 8) as u16,
                            self.numlock,
                            self.mod_state,
                        );

                        let bypassed = self.process_input_result(ret);

                        if bypassed {
                            self.key(time, key, state);
                        } else {
                            // If the key was not bypassed by IME, key repeat should be handled by the
                            // IME. Start waiting for the key hold timer event.
                            match self.repeat_state {
                                Some((info, ref mut press_state))
                                    if !press_state.is_pressing(key) =>
                                {
                                    let duration = Duration::from_millis(info.delay as u64);
                                    self.timer.set_timeout(&duration).unwrap();
                                    *press_state = PressState::Pressing {
                                        pressed_at: Instant::now(),
                                        is_repeating: false,
                                        key,
                                        wayland_time: time,
                                    }
                                }
                                _ => {}
                            }
                        }
                    } else {
                        self.key(time, key, state);
                    }
                } else {
                    if let Some((.., ref mut press_state)) = self.repeat_state {
                        if press_state.is_pressing(key) {
                            self.timer.disarm().unwrap();
                            *press_state = PressState::NotPressing;
                        }
                    }

                    self.key(time, key, state);
                }
            }
            KeyEvent::Modifiers {
                mods_depressed,
                mods_latched,
                mods_locked,
                group,
                ..
            } => {
                self.mod_state = 0;
                if mods_depressed & 0x1 != 0 {
                    self.mod_state |= ModifierState_SHIFT;
                }
                if mods_depressed & 0x4 != 0 {
                    self.mod_state |= ModifierState_CONTROL;
                }
                if mods_depressed & 0x8 != 0 {
                    self.mod_state |= ModifierState_ALT;
                }
                if mods_depressed & 0x40 != 0 {
                    self.mod_state |= ModifierState_SUPER;
                }

                self.numlock = mods_depressed & 0x10 != 0;

                if let Some(im_ctx) = &mut self.im_ctx {
                    im_ctx.modifiers(
                        self.serial,
                        mods_depressed,
                        mods_latched,
                        mods_locked,
                        group,
                    );
                }
            }
            KeyEvent::RepeatInfo { rate, delay } => {
                self.repeat_state = if rate == 0 {
                    None
                } else {
                    let info = RepeatInfo { rate, delay };
                    let press_state = self.repeat_state.map(|pair| pair.1);
                    Some((info, press_state.unwrap_or(PressState::NotPressing)))
                }
            }
            _ => {}
        }
    }

    pub fn handle_timer_ev(&mut self) -> std::io::Result<()> {
        // Read timer, this MUST be called or timer will be broken
        let overrun_count = self.timer.read()?;
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
            if !*is_repeating {
                if self
                    .keymap
                    .as_ref()
                    .map_or_else(|| true, |x| x.key_repeats(Keycode::new(key + 8)))
                {
                    // Start repeat
                    log::trace!("Start repeating {}", key);
                    let interval = &Duration::from_secs_f64(1.0 / info.rate as f64);
                    self.timer.set_timeout_interval(interval)?;
                    *is_repeating = true;
                }
            }

            let ev = KeyEvent::Key {
                serial: self.serial, // Is this fine?
                time: wayland_time + pressed_at.elapsed().as_millis() as u32,
                key,
                state: KeyState::Pressed,
            };
            self.handle_key_ev(ev);
        } else {
            log::warn!("Received timer event when it has never received RepeatInfo.");
        }
        Ok(())
    }

    pub fn activate(&mut self, im_ctx: Main<ZwpInputMethodContextV1>, keyboard: Main<WlKeyboard>) {
        self.engine.update_layout_state();
        if !self.engine_ready {
            if self.engine.check_ready() {
                let ret = self.engine.end_ready();
                self.process_input_result(ret);
                self.engine_ready = true;
            }
        }
        self.grab_activate = true;

        let filter = Filter::new(|ev, _filter, mut data| {
            let ctx = KimeContext::new_data(&mut data);

            match ev {
                Events::ImCtx { event, .. } => {
                    ctx.handle_im_ctx_ev(event);
                }
                Events::Key { event, .. } => {
                    ctx.handle_key_ev(event);
                }
                _ => {}
            }
        });

        im_ctx.assign(filter.clone());
        keyboard.assign(filter);

        self.im_ctx = Some(im_ctx);
        self.keyboard = Some(keyboard);
    }

    pub fn deactivate(&mut self) {
        // Focus lost, reset states
        if self.engine_ready {
            self.engine.reset();
        }
        self.grab_activate = false;

        // Input deactivated, stop repeating
        self.timer.disarm().unwrap();
        if let Some((_, ref mut press_state)) = self.repeat_state {
            *press_state = PressState::NotPressing
        }

        if let Some(im_ctx) = &mut self.im_ctx {
            im_ctx.destroy();
        }
        self.im_ctx = None;

        if let Some(keyboard) = &mut self.keyboard {
            if keyboard.as_ref().version() >= REQ_RELEASE_SINCE {
                keyboard.release();
            }
        }
        self.keyboard = None;
    }
}

pub fn run(
    display: &Display,
    event_queue: &mut EventQueue,
    globals: &GlobalManager,
) -> Result<(), Box<dyn Error>> {
    let im_filter = Filter::new(|ev, _filter, mut data| {
        let ctx = KimeContext::new_data(&mut data);
        match ev {
            Events::Im { event, .. } => match event {
                ImEvent::Activate { id: im_ctx } => {
                    let keyboard = im_ctx.grab_keyboard();
                    ctx.activate(im_ctx, keyboard);
                }
                ImEvent::Deactivate { .. } => {
                    ctx.deactivate();
                }
                _ => {}
            },
            _ => {}
        }
    });

    let im = globals.instantiate_exact::<ZwpInputMethodV1>(1)?;
    im.assign(im_filter);

    let mut timer = TimerFd::new(ClockId::Monotonic).expect("Initialize timer");

    let mut poll = Poll::new().expect("Initialize epoll()");
    let registry = poll.registry();

    const POLL_WAYLAND: Token = Token(0);
    registry
        .register(
            &mut SourceFd(&display.get_connection_fd()),
            POLL_WAYLAND,
            Interest::READABLE | Interest::WRITABLE,
        )
        .expect("Register wayland socket to the epoll()");

    const POLL_TIMER: Token = Token(1);
    registry
        .register(&mut timer, POLL_TIMER, Interest::READABLE)
        .expect("Register timer to the epoll()");

    // Initialize kime context
    let mut kime_ctx = KimeContext::new(timer);
    event_queue
        .sync_roundtrip(&mut kime_ctx, |_, _, _| ())
        .unwrap();

    log::info!("Server init success!");

    // Non-blocking event loop
    //
    // Reference:
    //   https://docs.rs/wayland-client/0.28.3/wayland_client/struct.EventQueue.html
    let mut events = MioEvents::with_capacity(1024);
    let stop_reason = 'main: loop {
        use std::io::ErrorKind;

        // Sleep until next event
        if let Err(e) = poll.poll(&mut events, None) {
            // Should retry on EINTR
            //
            // Reference:
            //   https://www.gnu.org/software/libc/manual/html_node/Interrupted-Primitives.html
            if e.kind() == ErrorKind::Interrupted {
                continue;
            }
            break Err(e);
        }

        for event in &events {
            match event.token() {
                POLL_WAYLAND => {}
                POLL_TIMER => {
                    if let Err(e) = kime_ctx.handle_timer_ev() {
                        break 'main Err(e);
                    }
                }
                _ => unreachable!(),
            }
        }

        // Perform read() only when it's ready, returns None when there're already pending events
        if let Some(guard) = event_queue.prepare_read() {
            if let Err(e) = guard.read_events() {
                // EWOULDBLOCK here means there's no new messages to read
                if e.kind() != ErrorKind::WouldBlock {
                    break Err(e);
                }
            }
        }

        if let Err(e) = event_queue.dispatch_pending(&mut kime_ctx, |_, _, _| {}) {
            break Err(e);
        }

        // Flush pending writes
        if let Err(e) = display.flush() {
            // EWOULDBLOCK here means there're so many to write, retry later
            if e.kind() != ErrorKind::WouldBlock {
                break Err(e);
            }
        }
    };

    match stop_reason {
        Ok(()) => {
            log::info!("Server finished gracefully");
            Ok(())
        }
        Err(e) => {
            log::error!("Server aborted due to IO Error: {}", e);
            Err(Box::from(e))
        }
    }
}
