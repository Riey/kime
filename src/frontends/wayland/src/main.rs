use wayland_client::Connection;

use kime_wayland::state::AppState;

fn main() {
    kime_version::cli_boilerplate!((),);

    let conn = Connection::connect_to_env().expect("Failed to connect wayland display");
    let display = conn.display();

    let mut event_queue = conn.new_event_queue::<AppState>();
    let qh = event_queue.handle();

    // Get registry to bind globals
    display.get_registry(&qh, ());

    // Create initial state
    let mut state = AppState::new(&conn);

    // Initial roundtrip to get globals
    event_queue.roundtrip(&mut state).unwrap();

    log::debug!(
        "Globals: v2={}, vk={}, seat={}, v1={}",
        state.has_input_method_v2(),
        state.has_vk_manager(),
        state.has_seat(),
        state.has_input_method_v1()
    );

    // Try v2 first, fall back to v1
    if state.has_input_method_v2() {
        log::info!("Using input_method_v2 protocol");
        if let Err(e) = state.setup_input_method_v2(&qh) {
            log::warn!("input_method_v2 setup failed: {}, trying v1", e);
            if state.has_input_method_v1() {
                state.setup_input_method_v1(&qh).unwrap();
            } else {
                log::error!("No input method protocol available");
                std::process::exit(1);
            }
        }
    } else if state.has_input_method_v1() {
        log::info!("Using input_method_v1 protocol");
        state.setup_input_method_v1(&qh).unwrap();
    } else {
        log::error!("No input method protocol available (no zwp_input_method_manager_v2, zwp_virtual_keyboard_manager_v1, or zwp_input_method_v1 found)");
        std::process::exit(1);
    }

    // Roundtrip after setup
    event_queue.roundtrip(&mut state).unwrap();

    log::info!("Server init success!");

    // Event loop
    state.run_event_loop(&conn, event_queue);
}
