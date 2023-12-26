use wayland_client::{Display, GlobalManager};

fn main() {
    kime_version::cli_boilerplate!((),);

    assert!(
        kime_engine_cffi::check_api_version(),
        "Engine version mismatched"
    );

    let display = Display::connect_to_env().expect("Failed to connect wayland display");
    let mut event_queue = display.create_event_queue();
    let attached_display = display.attach(event_queue.token());
    let globals = GlobalManager::new(&attached_display);

    event_queue.sync_roundtrip(&mut (), |_, _, _| ()).unwrap();

    let result = kime_wayland::input_method_v2::run(&display, &mut event_queue, &globals);

    if let Err(_) = result {
        kime_wayland::input_method_v1::run(&display, &mut event_queue, &globals).unwrap();
    }
}
