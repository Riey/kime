use x11rb::{
    connection::Connection,
    protocol::{ErrorKind, Event},
};
use xim::{x11rb::HasConnection, ServerError, XimConnections};

mod handler;
mod pe_window;

fn main() -> Result<(), ServerError> {
    kime_version::cli_boilerplate!(Ok(()),);

    assert!(
        kime_engine_cffi::check_api_version(),
        "Engine version mismatched"
    );

    let config = kime_engine_cffi::Config::load();

    let (conn, screen_num) = x11rb::xcb_ffi::XCBConnection::connect(None)?;
    let mut server = xim::x11rb::X11rbServer::init(conn, screen_num, "kime", xim::ALL_LOCALES)?;
    let mut connections = XimConnections::new();
    let mut handler = self::handler::KimeHandler::new(screen_num, config);

    loop {
        let e = server.conn().wait_for_event()?;
        if !server.filter_event(&e, &mut connections, &mut handler)? {
            match e {
                Event::Expose(e) => {
                    handler.expose(e.window);
                    server.conn().flush()?;
                }
                Event::ConfigureNotify(e) => {
                    handler.configure_notify(e);
                    server.conn().flush()?;
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
            }
        }
    }
}
