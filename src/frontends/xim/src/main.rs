use x11rb::{
    connection::Connection,
    protocol::{ErrorKind, Event},
};
use xim::{x11rb::HasConnection, XimConnections};

mod handler;
mod pe_window;

fn main() {
    kime_version::cli_boilerplate!((),);

    assert!(
        kime_engine_cffi::check_api_version(),
        "Engine version mismatched"
    );

    let config = kime_engine_cffi::Config::load();

    let (conn, screen_num) = x11rb::xcb_ffi::XCBConnection::connect(None).expect("Connect X");
    let mut server = xim::x11rb::X11rbServer::init(conn, screen_num, "kime", xim::ALL_LOCALES)
        .expect("Init XIM server");
    let mut connections = XimConnections::new();
    let mut handler = self::handler::KimeHandler::new(screen_num, config);

    loop {
        let e = server.conn().wait_for_event().expect("Wait event");
        match server.filter_event(&e, &mut connections, &mut handler) {
            // event has filtered
            Ok(true) => {}
            // event hasn't filtered
            Ok(false) => match e {
                Event::Expose(e) => {
                    handler.expose(e.window);
                    server.conn().flush().expect("Flush connection");
                }
                Event::ConfigureNotify(e) => {
                    handler.configure_notify(e);
                    server.conn().flush().expect("Flush connection");
                }
                Event::UnmapNotify(..) => {}
                Event::DestroyNotify(..) => {}
                Event::MappingNotify(..) => {}
                Event::Error(x11rb::x11_utils::X11Error {
                    error_kind: ErrorKind::RenderPicture,
                    ..
                }) => {}
                e => {
                    log::trace!("Unfiltered event: {:?}", e);
                }
            },
            Err(err) => {
                // Don't stop server just logging
                log::error!("ServerError occured while process event: {}", err);
            }
        }
    }
}
