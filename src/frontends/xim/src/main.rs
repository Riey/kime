use x11rb::{
    connection::Connection,
    protocol::{ErrorKind, Event},
};
use xim::{x11rb::HasConnection, ServerError, XimConnections};

mod handler;
mod pe_window;

fn main() -> Result<(), ServerError> {
    let mut args = pico_args::Arguments::from_env();

    if args.contains(["-h", "--help"]) {
        println!("-h or --help: show help");
        println!("-v or --version: show version");
        println!("--verbose: more verbose log");
        return Ok(());
    }

    if args.contains(["-v", "--version"]) {
        kime_version::print_version!();
        return Ok(());
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

    log::info!("Start xim server version: {}", env!("CARGO_PKG_VERSION"));

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
