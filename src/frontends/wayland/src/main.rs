use std::time::Duration;

use wayland_client::{
    event_enum,
    protocol::{wl_keyboard::KeyState, wl_seat::WlSeat},
    DispatchData, Display, Filter, GlobalManager, Main,
};

use zwp_input_method::input_method_unstable_v2::{
    zwp_input_method_keyboard_grab_v2::{Event as KeyEvent, ZwpInputMethodKeyboardGrabV2},
    zwp_input_method_manager_v2::ZwpInputMethodManagerV2,
    zwp_input_method_v2::{Event as ImEvent, ZwpInputMethodV2},
};
use zwp_virtual_keyboard::virtual_keyboard_unstable_v1::{
    zwp_virtual_keyboard_manager_v1::ZwpVirtualKeyboardManagerV1,
    zwp_virtual_keyboard_v1::ZwpVirtualKeyboardV1,
};

use kime_engine_cffi::{
    Config, InputEngine, InputResultType, ModifierState, MODIFIER_ALT, MODIFIER_CONTROL,
    MODIFIER_SHIFT, MODIFIER_SUPER,
};

use mio::{unix::SourceFd, Events as MioEvents, Interest, Poll, Token};
use mio_timerfd::{ClockId, TimerFd};

event_enum! {
    Events |
    Key => ZwpInputMethodKeyboardGrabV2,
    Im => ZwpInputMethodV2
}

struct InputMethodState {
    activate: bool,
    deactivate: bool,
}

impl Default for InputMethodState {
    fn default() -> Self {
        Self {
            activate: false,
            deactivate: false,
        }
    }
}

#[derive(Clone, Copy)]
struct RepeatInfo {
    /// The rate of repeating keys in characters per second
    rate: i32,
    /// Delay in milliseconds since key down until repeating starts
    delay: i32,
}

#[derive(Clone, Copy)]
enum PressState {
    /// User is pressing no key, or user lifted last pressed key. But kime-wayland is ready for key
    /// long-press.
    NotPressing,
    /// User just started pressing a key. Soon, key repeating will be begin.
    NotRepeatingYet {
        /// Key code used by wayland
        key: u32,
        /// Timestamp with millisecond granularity used by wayland. Their base is undefined, so
        /// they can't be compared against system time (as obtained with clock_gettime or
        /// gettimeofday). They can be compared with each other though, and for instance be used to
        /// identify sequences of button presses as double or triple clicks.
        ///
        /// #### Reference
        /// - https://wayland.freedesktop.org/docs/html/ch04.html#sect-Protocol-Input
        time: u32,
    },
    /// User have pressed a key for a long enough time. Key repeating is happening right now.
    Repeating {
        /// Key code used by wayland
        key: u32,
        /// Timestamp with millisecond granularity used by wayland. Their base is undefined, so
        /// they can't be compared against system time (as obtained with clock_gettime or
        /// gettimeofday). They can be compared with each other though, and for instance be used to
        /// identify sequences of button presses as double or triple clicks.
        ///
        /// #### Reference
        /// - https://wayland.freedesktop.org/docs/html/ch04.html#sect-Protocol-Input
        time: u32,
    },
}

impl PressState {
    fn is_pressing(&self, query_key: u32) -> bool {
        if let PressState::NotRepeatingYet { key, .. } | PressState::Repeating { key, .. } = self {
            *key == query_key
        } else {
            false
        }
    }
}

struct KimeContext {
    config: Config,
    engine: InputEngine,
    mod_state: ModifierState,
    current_state: InputMethodState,
    pending_state: InputMethodState,
    vk: Main<ZwpVirtualKeyboardV1>,
    im: Main<ZwpInputMethodV2>,
    grab_kb: Option<Main<ZwpInputMethodKeyboardGrabV2>>,
    keymap_init: bool,
    serial: u32,
    // Have to consern Multi seats?

    // Key repeat contexts
    timer: TimerFd,
    repeat_state: Option<(RepeatInfo, PressState)>,
}

impl Drop for KimeContext {
    fn drop(&mut self) {
        self.vk.destroy();
        self.im.destroy();
        if let Some(kb) = self.grab_kb.take() {
            kb.release();
        }
    }
}

impl KimeContext {
    pub fn new(vk: Main<ZwpVirtualKeyboardV1>, im: Main<ZwpInputMethodV2>, timer: TimerFd) -> Self {
        Self {
            config: Config::new(),
            engine: InputEngine::new(),
            mod_state: 0,
            current_state: InputMethodState::default(),
            pending_state: InputMethodState::default(),
            serial: 0,
            keymap_init: false,
            vk,
            im,
            grab_kb: None,

            timer,
            repeat_state: None,
        }
    }

    pub fn new_data<'a>(data: &'a mut DispatchData) -> &'a mut Self {
        data.get::<Self>().unwrap()
    }

    fn commit(&mut self) {
        self.im.commit(self.serial);
        self.serial += 1;
    }

    fn commit_ch(&mut self, ch: char) {
        self.im.commit_string(ch.to_string());
    }

    fn commit_ch2(&mut self, ch1: char, ch2: char) {
        let mut buf = String::with_capacity(ch1.len_utf8() + ch2.len_utf8());
        buf.push(ch1);
        buf.push(ch2);
        self.im.commit_string(buf);
    }

    fn clear_preedit(&mut self) {
        self.im.set_preedit_string(String::new(), -1, -1);
    }

    fn preedit_ch(&mut self, ch: char) {
        self.im
            .set_preedit_string(ch.to_string(), 0, ch.len_utf8() as _);
    }

    pub fn handle_im_ev(&mut self, ev: ImEvent, filter: &Filter<Events>) {
        match ev {
            ImEvent::Activate => {
                self.pending_state.activate = true;
            }
            ImEvent::Deactivate => {
                self.pending_state.deactivate = true;
            }
            ImEvent::Unavailable => {
                self.vk.destroy();
                self.im.destroy();
            }
            ImEvent::Done => {
                if !self.current_state.activate && self.pending_state.activate {
                    self.engine.update_hangul_state();
                    let kb = self.im.grab_keyboard();
                    kb.assign(filter.clone());
                    self.grab_kb = Some(kb);
                } else if !self.current_state.deactivate && self.pending_state.deactivate {
                    // Focus lost, reset states
                    self.engine.reset();
                    if let Some(kb) = self.grab_kb.take() {
                        kb.release();
                    }
                    self.timer.disarm().unwrap();
                    self.repeat_state = None
                }
                self.current_state = std::mem::take(&mut self.pending_state);
            }
            _ => {}
        }
    }

    pub fn handle_key_ev(&mut self, ev: KeyEvent) {
        match ev {
            KeyEvent::Keymap { fd, format, size } => {
                if !self.keymap_init {
                    self.vk.keymap(format as _, fd, size);
                    self.keymap_init = true;
                }

                unsafe {
                    libc::close(fd);
                }
            }
            KeyEvent::Key {
                state, key, time, ..
            } => {
                // NOTE: Never read `serial` of KeyEvent. You should rely on serial of KimeContext
                if state == KeyState::Pressed {
                    // Start waiting for the key hold timer event
                    if let Some((info, ref mut press_state)) = self.repeat_state {
                        if !press_state.is_pressing(key) {
                            let duration = Duration::from_millis(info.delay as u64);
                            self.timer.set_timeout(&duration).unwrap();
                            *press_state = PressState::NotRepeatingYet { key, time };
                        }
                    }

                    let mut bypass = false;
                    let ret = self
                        .engine
                        .press_key(&self.config, (key + 8) as u16, self.mod_state);
                    log::trace!("ret: {:#?}", ret);

                    match ret.ty {
                        InputResultType::ToggleHangul => {
                            self.engine.update_hangul_state();
                        }
                        InputResultType::Bypass => bypass = true,
                        InputResultType::CommitBypass => {
                            self.commit_ch(ret.char1);
                            bypass = true;
                        }
                        InputResultType::Commit => {
                            self.commit_ch(ret.char1);
                        }
                        InputResultType::Preedit => {
                            self.preedit_ch(ret.char1);
                        }
                        InputResultType::CommitPreedit => {
                            self.commit_ch(ret.char1);
                            self.preedit_ch(ret.char2);
                        }
                        InputResultType::CommitCommit => {
                            self.commit_ch2(ret.char1, ret.char2);
                        }
                        InputResultType::ClearPreedit => {
                            self.clear_preedit();
                        }
                    }

                    self.commit();

                    if bypass {
                        self.vk.key(time, key, state as _);
                    }
                } else {
                    // If user released the last pressed key, clear the timer and state
                    if let Some((.., ref mut press_state)) = self.repeat_state {
                        if press_state.is_pressing(key) {
                            self.timer.disarm().unwrap();
                            *press_state = PressState::NotPressing;
                        }
                    }

                    self.vk.key(time, key, state as _);
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
                    self.mod_state |= MODIFIER_SHIFT;
                }
                if mods_depressed & 0x4 != 0 {
                    self.mod_state |= MODIFIER_CONTROL;
                }
                if mods_depressed & 0x8 != 0 {
                    self.mod_state |= MODIFIER_ALT;
                }
                if mods_depressed & 0x40 != 0 {
                    self.mod_state |= MODIFIER_SUPER;
                }
                self.vk
                    .modifiers(mods_depressed, mods_latched, mods_locked, group);
            }
            KeyEvent::RepeatInfo { rate, delay } => {
                self.repeat_state = if rate == 0 {
                    // Zero rate means disabled repeat
                    //
                    // Reference:
                    //   https://github.com/swaywm/wlroots/blob/3d46d3f7/protocol/input-method-unstable-v2.xml#L444-L455
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
        self.timer.read()?;

        if let Some((info, ref mut press_state)) = self.repeat_state {
            let (key, time) = match press_state {
                PressState::NotPressing => {
                    log::warn!("Received timer event while pressing no key.");
                    return Ok(());
                }
                PressState::NotRepeatingYet { key, time } => {
                    // Start repeat
                    log::trace!("Start repeating {}", key);
                    let interval = &Duration::from_secs_f64(1.0 / info.rate as f64);
                    self.timer.set_timeout_interval(interval)?;

                    let (key, time) = (*key, *time);
                    *press_state = PressState::Repeating { key, time };
                    (key, time)
                }
                PressState::Repeating { key, time } => {
                    log::trace!("Keep repeating {}", key);
                    (*key, *time)
                }
            };

            // Emit key repeat event
            let ev = KeyEvent::Key {
                serial: self.serial,
                // NOTE: Not sure if this time should be the time when the key was
                // initially pressed, or the time of this KeyEvent
                time,
                key,
                state: KeyState::Pressed,
            };
            self.serial += 1;
            self.handle_key_ev(ev);
        } else {
            log::warn!("Received timer event when it has never received RepeatInfo.");
        }

        Ok(())
    }
}

fn main() {
    let mut args = pico_args::Arguments::from_env();

    if args.contains(["-h", "--help"]) {
        println!("-h or --help: show help");
        println!("-v or --version: show version");
        println!("--verbose: more verbose log");
        return;
    }

    if args.contains(["-v", "--version"]) {
        kime_version::print_version!();
        return;
    }

    let mut log_level = if cfg!(debug_assertions) {
        log::LevelFilter::Trace
    } else {
        log::LevelFilter::Info
    };

    if args.contains("--verbose") {
        log_level = log::LevelFilter::Trace;
    }

    simplelog::SimpleLogger::init(log_level, simplelog::ConfigBuilder::new().build()).ok();

    log::info!(
        "Start wayland im server version: {}",
        env!("CARGO_PKG_VERSION")
    );

    let display = Display::connect_to_env().expect("Failed to connect wayland display");
    let mut event_queue = display.create_event_queue();
    let attached_display = display.attach(event_queue.token());
    let globals = GlobalManager::new(&attached_display);

    event_queue.sync_roundtrip(&mut (), |_, _, _| ()).unwrap();

    let seat = globals.instantiate_exact::<WlSeat>(1).expect("Load Seat");
    let im_manager = globals
        .instantiate_exact::<ZwpInputMethodManagerV2>(1)
        .expect("Load InputManager");
    let vk_manager = globals
        .instantiate_exact::<ZwpVirtualKeyboardManagerV1>(1)
        .expect("Load VirtualKeyboardManager");

    let filter = Filter::new(|ev, filter, mut data| {
        let ctx = KimeContext::new_data(&mut data);

        match ev {
            Events::Key { event, .. } => {
                ctx.handle_key_ev(event);
            }
            Events::Im { event, .. } => {
                ctx.handle_im_ev(event, filter);
            }
        }
    });

    let vk = vk_manager.create_virtual_keyboard(&seat);
    let im = im_manager.get_input_method(&seat);
    im.assign(filter);

    // Initialize timer
    let mut timer = TimerFd::new(ClockId::Monotonic).expect("Initialize timer");

    // Initialize epoll() object
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
    let mut kime_ctx = KimeContext::new(vk, im, timer);
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
        Ok(()) => log::info!("Server finished gracefully"),
        Err(e) => log::error!("Server aborted due to IO Error: {}", e),
    }
}
