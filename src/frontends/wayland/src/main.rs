use wayland_client::{
    event_enum,
    protocol::{wl_keyboard::KeyState, wl_seat::WlSeat},
    DispatchData, Display, Filter, GlobalManager, Main,
};

use std::sync::atomic::{AtomicBool, Ordering};
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
    Config, InputEngine, InputResultType, ModifierState, MODIFIER_CONTROL, MODIFIER_SHIFT,
    MODIFIER_SUPER,
};

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
    pub fn new(vk: Main<ZwpVirtualKeyboardV1>, im: Main<ZwpInputMethodV2>) -> Self {
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
                    if let Some(c) = self.engine.reset() {
                        self.commit_ch(c);
                        self.commit();
                    }
                    if let Some(kb) = self.grab_kb.take() {
                        kb.release();
                    }
                }
                self.current_state = std::mem::take(&mut self.pending_state);
            }
            ImEvent::TextChangeCause { .. } | ImEvent::SurroundingText { .. } | _ => {}
        }
    }

    pub fn handle_key_ev(&mut self, ev: KeyEvent) {
        match ev {
            KeyEvent::Keymap { fd, format, size } => {
                if !self.keymap_init {
                    self.vk.keymap(format as _, fd, size);
                    unsafe {
                        libc::close(fd);
                    }
                    self.keymap_init = true;
                }
            }
            KeyEvent::Key {
                state, key, time, ..
            } => {
                if state == KeyState::Pressed {
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
                    self.vk.key(time, key, state as _);
                }

                // TODO: repeat key
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
                if mods_depressed & 0x40 != 0 {
                    self.mod_state |= MODIFIER_SUPER;
                }
                self.vk
                    .modifiers(mods_depressed, mods_latched, mods_locked, group);
            }
            KeyEvent::RepeatInfo { .. } => {
                // TODO: repeat key
            }
            _ => {}
        }
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

    static STOP: AtomicBool = AtomicBool::new(false);

    ctrlc::set_handler(|| {
        STOP.store(true, Ordering::Relaxed);
    })
    .expect("Set handler");

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
    let attached_display = (*display).clone().attach(event_queue.token());
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

    let mut kime_ctx = KimeContext::new(vk, im);

    event_queue
        .sync_roundtrip(&mut kime_ctx, |_, _, _| ())
        .unwrap();

    log::info!("Server init success!");

    while !STOP.load(Ordering::Relaxed) {
        // ignore unfiltered messages
        match event_queue.dispatch(&mut kime_ctx, |_, _, _| ()) {
            Ok(_) => {}
            Err(err) => {
                if err.kind() == std::io::ErrorKind::WouldBlock {
                    continue;
                }

                if err.kind() != std::io::ErrorKind::Interrupted {
                    log::error!("IO Error: {}", err);
                }

                break;
            }
        }
    }

    log::info!("End server...");
}
