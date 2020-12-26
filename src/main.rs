use x11rb::connection::Connection;
use xim::{ServerError, XimConnections};

mod engine;
mod handler;

fn main() -> Result<(), ServerError> {
    pretty_env_logger::init();
    let (conn, screen_num) = x11rb::rust_connection::RustConnection::connect(None)?;
    let screen = &conn.setup().roots[screen_num];
    let mut server = xim::x11rb::X11rbServer::init(&conn, screen, "test_server")?;
    let mut connections = XimConnections::new();
    let mut handler = self::handler::KimeHandler::new();

    loop {
        let e = conn.wait_for_event()?;
        server.filter_event(&e, &mut connections, &mut handler)?;
    }
}
