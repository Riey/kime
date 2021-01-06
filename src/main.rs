use x11rb::{
    connection::Connection,
    protocol::{ErrorKind, Event},
};
use xim::{x11rb::HasConnection, ServerError, XimConnections};

mod engine;
mod handler;

fn main() -> Result<(), ServerError> {
    pretty_env_logger::init();
    let (conn, screen_num) = x11rb::xcb_ffi::XCBConnection::connect(None)?;
    let mut server = xim::x11rb::X11rbServer::init(conn, screen_num, "kime")?;
    let mut connections = XimConnections::new();
    let mut handler = self::handler::KimeHandler::new(screen_num);

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
