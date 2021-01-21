use wayland_client::{
    event_enum,
    protocol::{wl_compositor, wl_keyboard, wl_seat},
    Display, Filter, GlobalManager,
};
use wayland_protocols::unstable::input_method::v1::client::zwp_input_panel_surface_v1;
use wayland_protocols::unstable::text_input::v3::client::zwp_text_input_v3;

use zwp_input_method::input_method_unstable_v2::{
    zwp_input_method_keyboard_grab_v2, zwp_input_method_manager_v2, zwp_input_method_v2,
    zwp_input_popup_surface_v2,
};
use zwp_virtual_keyboard::virtual_keyboard_unstable_v1::{
    zwp_virtual_keyboard_manager_v1, zwp_virtual_keyboard_v1,
};

#[derive(Default)]
struct SurroundingText {
    text: String,
    cursor: usize,
    anchor: usize,
}

struct InputMethodState {
    change_cause: zwp_text_input_v3::ChangeCause,
    hint: zwp_text_input_v3::ContentHint,
    purpose: zwp_text_input_v3::ContentPurpose,
    surrounding_text: SurroundingText,
}

impl Default for InputMethodState {
    fn default() -> Self {
        Self {
            change_cause: zwp_text_input_v3::ChangeCause::Other,
            hint: zwp_text_input_v3::ContentHint::empty(),
            purpose: zwp_text_input_v3::ContentPurpose::Normal,
            surrounding_text: SurroundingText::default(),
        }
    }
}

event_enum! {
    Events |
    Keyboard => wl_keyboard::WlKeyboard,
    Im => zwp_input_method_v2::ZwpInputMethodV2
}

fn main() {
    let display = Display::connect_to_env().expect("Failed to connect wayland display");
    let mut event_queue = display.create_event_queue();
    let attached_display = (*display).clone().attach(event_queue.token());
    let globals = GlobalManager::new(&attached_display);

    event_queue
        .sync_roundtrip(&mut (), |_, _, _| unreachable!())
        .unwrap();

    let compositor = globals
        .instantiate_exact::<wl_compositor::WlCompositor>(1)
        .expect("Load compositor");
    let surface = compositor.create_surface();
    let input_manager = globals
        .instantiate_exact::<zwp_input_method_manager_v2::ZwpInputMethodManagerV2>(1)
        .expect("Load InputManager");
    let virtual_keyboard_manager = globals
        .instantiate_exact::<zwp_virtual_keyboard_manager_v1::ZwpVirtualKeyboardManagerV1>(1)
        .expect("Load VirtualKeyboardManager");
    let seat = globals
        .instantiate_exact::<wl_seat::WlSeat>(1)
        .expect("Load seat");
    let mut vk = virtual_keyboard_manager.create_virtual_keyboard(&seat);
    seat.get_keyboard().quick_assign(move |_key, event, _data| {
        eprintln!("{:?}", event);

        match event {
            _ => {}
        }
    });
    let im = input_manager.get_input_method(&seat);
    let mut serial = 0;
    let mut active = false;
    let mut pending_active = false;
    let mut kb_grab = None;
    let mut pending_state = InputMethodState::default();
    let mut current_state = InputMethodState::default();

    im.quick_assign(move |im, event, _| match event {
        zwp_input_method_v2::Event::Activate => {
            pending_active = true;
        }
        zwp_input_method_v2::Event::Deactivate => {
            pending_active = false;
        }
        zwp_input_method_v2::Event::ContentType { hint, purpose } => {
            pending_state.hint = hint;
            pending_state.purpose = purpose;
        }
        zwp_input_method_v2::Event::TextChangeCause { cause } => {
            pending_state.change_cause = cause;
        }
        zwp_input_method_v2::Event::SurroundingText {
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
        zwp_input_method_v2::Event::Unavailable => {
            im.destroy();
        }
        zwp_input_method_v2::Event::Done => {
            let prev_active = active;
            serial += 1;
            current_state = std::mem::take(&mut pending_state);
            active = pending_active;

            if active && !prev_active {
                let kb = im.grab_keyboard();
                kb.quick_assign(move |_kb, event, _| {
                    eprintln!("{:?}", event);
                });
                eprintln!("Set kb_grab");
                im.commit(serial);
                kb_grab = Some(dbg!(kb));
            } else if !active && prev_active {
                kb_grab = None;
            }
        }
        _ => {}
    });

    event_queue.sync_roundtrip(&mut (), |_, _, _| ()).unwrap();
    loop {
        // ignore unfiltered messages
        event_queue.dispatch(&mut (), |_, _, _| ()).unwrap();
    }
}
