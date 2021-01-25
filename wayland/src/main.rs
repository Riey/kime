use wayland_client::{
    protocol::{wl_keyboard::KeyState, wl_seat},
    Display, GlobalManager,
};
use wayland_protocols::unstable::text_input::v3::client::zwp_text_input_v3;

use std::cell::Cell;
use std::num::NonZeroI32;
use zwp_input_method::input_method_unstable_v2::{
    zwp_input_method_keyboard_grab_v2::Event as KeyEvent,
    zwp_input_method_manager_v2::ZwpInputMethodManagerV2, zwp_input_method_v2::Event as ImEvent,
};
use zwp_virtual_keyboard::virtual_keyboard_unstable_v1::zwp_virtual_keyboard_manager_v1::ZwpVirtualKeyboardManagerV1;

use kime_engine_cffi::{
    Config, InputEngine, InputResultType, MODIFIER_CONTROL, MODIFIER_SHIFT, MODIFIER_SUPER,
};

#[derive(Default)]
struct RepeatInfo {
    rate: Option<NonZeroI32>,
    delay: Option<NonZeroI32>,
}

impl RepeatInfo {
    pub fn new(rate: i32, delay: i32) -> Self {
        Self {
            rate: NonZeroI32::new(rate),
            delay: NonZeroI32::new(delay),
        }
    }
}

#[derive(Default)]
struct SurroundingText {
    text: String,
    cursor: usize,
    anchor: usize,
}

struct InputMethodState {
    activate: bool,
    deactivate: bool,
    change_cause: zwp_text_input_v3::ChangeCause,
    hint: zwp_text_input_v3::ContentHint,
    purpose: zwp_text_input_v3::ContentPurpose,
    surrounding_text: SurroundingText,
}

impl Default for InputMethodState {
    fn default() -> Self {
        Self {
            activate: false,
            deactivate: false,
            change_cause: zwp_text_input_v3::ChangeCause::Other,
            hint: zwp_text_input_v3::ContentHint::empty(),
            purpose: zwp_text_input_v3::ContentPurpose::Normal,
            surrounding_text: SurroundingText::default(),
        }
    }
}

fn main() {
    let mut args = pico_args::Arguments::from_env();

    if args.contains("--version") {
        println!("kime-xim: {}", env!("CARGO_PKG_VERSION"));

        return;
    }

    let mut log_level = if cfg!(debug_assertions) {
        log::LevelFilter::Trace
    } else {
        log::LevelFilter::Info
    };

    if args.contains("--log") {
        log_level = log::LevelFilter::Trace;
    }

    simplelog::SimpleLogger::init(log_level, simplelog::ConfigBuilder::new().build()).ok();

    log::info!(
        "Start wayland im server version: {}",
        env!("CARGO_PKG_VERSION")
    );

    let config: &'static Config = Box::leak(Box::new(Config::new()));
    let serial: &'static Cell<u32> = Box::leak(Box::new(Cell::new(0)));

    let display = Display::connect_to_env().expect("Failed to connect wayland display");
    let mut event_queue = display.create_event_queue();
    let attached_display = (*display).clone().attach(event_queue.token());
    let globals = GlobalManager::new(&attached_display);

    event_queue
        .sync_roundtrip(&mut (), |_, _, _| unreachable!())
        .unwrap();

    let input_manager = globals
        .instantiate_exact::<ZwpInputMethodManagerV2>(1)
        .expect("Load InputManager");
    let virtual_keyboard_manager = globals
        .instantiate_exact::<ZwpVirtualKeyboardManagerV1>(1)
        .expect("Load VirtualKeyboardManager");
    let seat = globals
        .instantiate_exact::<wl_seat::WlSeat>(1)
        .expect("Load seat");
    let vk = virtual_keyboard_manager.create_virtual_keyboard(&seat);
    let im = input_manager.get_input_method(&seat);
    let mut kb_grab = None;
    let mut pending_state = InputMethodState::default();
    let mut current_state = InputMethodState::default();
    let mut engine = InputEngine::new();

    im.quick_assign(move |im, event, mut data| {
        log::trace!("Im event: {:?}", event);

        match event {
            ImEvent::Activate => {
                pending_state.activate = true;
            }
            ImEvent::Deactivate => {
                pending_state.deactivate = true;
            }
            ImEvent::ContentType { hint, purpose } => {
                pending_state.hint = hint;
                pending_state.purpose = purpose;
            }
            ImEvent::TextChangeCause { cause } => {
                pending_state.change_cause = cause;
            }
            ImEvent::SurroundingText {
                text,
                anchor,
                cursor,
            } => {
                pending_state.surrounding_text = SurroundingText {
                    text,
                    cursor: cursor as usize,
                    anchor: anchor as usize,
                };
            }
            ImEvent::Unavailable => {
                im.destroy();
            }
            ImEvent::Done => {
                serial.set(serial.get() + 1);

                if !current_state.activate && pending_state.activate {
                    let kb = im.grab_keyboard();
                    let vk = vk.clone();
                    let mut keymap_init = false;
                    let mut kime_state = 0;
                    let mut repeat_info = RepeatInfo::default();
                    kb.quick_assign(move |_kb, event, mut data| {
                        let engine = data.get::<InputEngine>().unwrap();
                        match event {
                            KeyEvent::Keymap { fd, format, size } => {
                                if !keymap_init {
                                    vk.keymap(format as _, fd, size);
                                    unsafe {
                                        libc::close(fd);
                                    }
                                    keymap_init = true;
                                }
                            }
                            KeyEvent::Key {
                                state, key, time, ..
                            } => {
                                if state == KeyState::Pressed {
                                    let mut bypass = false;
                                    let ret =
                                        engine.press_key(config, (key + 8) as u16, kime_state);
                                    log::trace!("ret: {:#?}", ret);

                                    match ret.ty {
                                        InputResultType::Consume => {}
                                        InputResultType::Bypass => bypass = true,
                                        InputResultType::CommitBypass => {
                                            im.commit_string(ret.char1.to_string());
                                            bypass = true;
                                        }
                                        InputResultType::Commit => {
                                            im.commit_string(ret.char1.to_string());
                                        }
                                        InputResultType::Preedit => {
                                            im.set_preedit_string(
                                                ret.char1.to_string(),
                                                0,
                                                ret.char1.len_utf8() as _,
                                            );
                                        }
                                        InputResultType::CommitPreedit => {
                                            im.commit_string(ret.char1.to_string());
                                            im.set_preedit_string(
                                                ret.char2.to_string(),
                                                0,
                                                ret.char2.len_utf8() as _,
                                            );
                                        }
                                        InputResultType::CommitCommit => {
                                            im.commit_string(ret.char1.to_string());
                                            im.commit_string(ret.char2.to_string());
                                        }
                                        InputResultType::ClearPreedit => {
                                            im.set_preedit_string(String::new(), -1, -1);
                                        }
                                    }

                                    im.commit(serial.get());

                                    if bypass {
                                        vk.key(time, key, state as _);
                                    }
                                } else {
                                    vk.key(time, key, state as _);
                                }

                                // TODO repeat key
                            }
                            KeyEvent::Modifiers {
                                mods_depressed,
                                mods_latched,
                                mods_locked,
                                group,
                                ..
                            } => {
                                kime_state = 0;
                                if mods_depressed & 0x1 != 0 {
                                    log::trace!("Set SHIFT");
                                    kime_state |= MODIFIER_SHIFT;
                                }
                                if mods_depressed & 0x4 != 0 {
                                    log::trace!("Set CONTROL");
                                    kime_state |= MODIFIER_CONTROL;
                                }
                                if mods_depressed & 0x40 != 0 {
                                    log::trace!("Set SUPER");
                                    kime_state |= MODIFIER_SUPER;
                                }
                                vk.modifiers(mods_depressed, mods_latched, mods_locked, group);
                            }
                            KeyEvent::RepeatInfo { delay, rate } => {
                                repeat_info = RepeatInfo::new(rate, delay);
                            }
                            _ => {}
                        }
                    });
                    kb_grab = Some(kb);
                } else if pending_state.deactivate {
                    log::trace!("Focus out");

                    let engine = data.get::<InputEngine>().unwrap();

                    if let Some(c) = engine.reset() {
                        // This act wrong now so disable temporary it will cause end letter bug
                        // im.commit_string(c.to_string());
                        // im.commit(serial.get());
                    }
                    if let Some(grab) = kb_grab.take() {
                        grab.release();
                    }
                }

                current_state = std::mem::take(&mut pending_state);
            }
            _ => {}
        }
    });

    event_queue.sync_roundtrip(&mut (), |_, _, _| ()).unwrap();

    log::info!("Server init success!");

    loop {
        // ignore unfiltered messages
        event_queue.dispatch(&mut engine, |_, _, _| ()).unwrap();
    }
}
